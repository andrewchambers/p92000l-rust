use super::fcall;
use super::fcall::{Fcall, TaggedFcall};
use crossbeam_channel as channel;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;

fn err_other(msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, msg)
}

fn err_not_dir() -> std::io::Error {
    err_other("not a directory")
}

fn err_io() -> std::io::Error {
    err_other("io error")
}

fn err_io_result<R, E>(_e: E) -> Result<R, std::io::Error> {
    Err(err_io())
}

fn err_unexpected_response() -> std::io::Error {
    err_other("unexpected response from server")
}

struct Fid {
    id: u32,
    set: Arc<Mutex<HashSet<u32>>>,
}

impl Drop for Fid {
    fn drop(&mut self) {
        self.set.lock().unwrap().remove(&self.id);
    }
}

struct Fidset {
    set: Arc<Mutex<HashSet<u32>>>,
    next: Mutex<u32>,
}

impl Fidset {
    fn new() -> Fidset {
        Fidset {
            set: Arc::new(Mutex::new(HashSet::new())),
            next: Mutex::new(fcall::NOFID),
        }
    }

    fn fresh(&self) -> Option<Fid> {
        let mut next = self.next.lock().unwrap();
        let mut set = self.set.lock().unwrap();
        if set.len() == (fcall::NOFID - 1) as usize {
            return None;
        }
        loop {
            if *next == fcall::NOFID {
                *next = 0;
            } else {
                *next += 1;
            }
            if !set.contains(&next) {
                let id = *next;
                set.insert(id);
                return Some(Fid {
                    id,
                    set: self.set.clone(),
                });
            }
        }
    }
}

struct InflightFcallsState {
    disconnected: bool,
    map: HashMap<u16, channel::Sender<Fcall<'static>>>,
    next_tag: u16,
}

impl InflightFcallsState {
    fn new() -> InflightFcallsState {
        InflightFcallsState {
            disconnected: false,
            map: HashMap::new(),
            next_tag: fcall::NOTAG,
        }
    }

    fn mark_disconnected(&mut self) {
        // Trigger EIO for listeners.
        self.map.clear();
        self.disconnected = true;
    }

    fn add(&mut self, respond_to: channel::Sender<Fcall<'static>>) -> Result<u16, std::io::Error> {
        if self.disconnected {
            return Err(err_other("disconnected"));
        };

        if self.map.len() == (fcall::NOTAG - 1) as usize {
            return Err(err_other("too many in flight fcalls"));
        }

        loop {
            if self.next_tag == fcall::NOTAG {
                self.next_tag = 0;
            } else {
                self.next_tag += 1;
            }
            if !self.map.contains_key(&self.next_tag) {
                let tag = self.next_tag;
                self.map.insert(tag, respond_to);
                return Ok(tag);
            }
        }
    }

    fn remove(&mut self, tag: u16) -> Option<channel::Sender<Fcall<'static>>> {
        self.map.remove(&tag)
    }
}

#[derive(Clone)]
struct InflightFcalls {
    state: Arc<Mutex<InflightFcallsState>>,
}

impl InflightFcalls {
    fn new() -> InflightFcalls {
        InflightFcalls {
            state: Arc::new(Mutex::new(InflightFcallsState::new())),
        }
    }

    fn mark_disconnected(&self) {
        self.state.lock().unwrap().mark_disconnected()
    }

    fn add(&self, respond_to: channel::Sender<Fcall<'static>>) -> Result<u16, std::io::Error> {
        self.state.lock().unwrap().add(respond_to)
    }

    fn remove(&self, tag: u16) -> Option<channel::Sender<Fcall<'static>>> {
        self.state.lock().unwrap().remove(tag)
    }
}

struct ClientWriteState {
    w: TcpStream,
    buf: Vec<u8>,
}

struct DotlClientState {
    msize: u32,
    fids: Fidset,
    fcalls: InflightFcalls,
    write_state: Mutex<ClientWriteState>,
    read_worker_handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for DotlClientState {
    fn drop(&mut self) {
        {
            let write_state = self.write_state.lock().unwrap();
            write_state.w.shutdown(std::net::Shutdown::Both).unwrap();
            drop(write_state)
        }
        if let Some(read_worker_handle) = self.read_worker_handle.take() {
            let _ = read_worker_handle.join();
        }
    }
}

#[derive(Clone)]
pub struct DotlClient {
    state: Arc<DotlClientState>,
}

impl DotlClient {
    pub fn tcp(conn: TcpStream, bufsize: usize) -> Result<DotlClient, std::io::Error> {
        let mut r = conn;
        let mut w = r.try_clone()?;

        const MIN_MSIZE: u32 = 4096 + fcall::READDIRHDRSZ;
        let mut bufsize = bufsize.max(MIN_MSIZE as usize).min(u32::MAX as usize);
        let mut wbuf = Vec::with_capacity(bufsize);
        let mut rbuf = Vec::with_capacity(bufsize);

        fcall::write(
            &mut w,
            &mut wbuf,
            &TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Tversion(fcall::Tversion {
                    msize: bufsize.min(u32::MAX as usize) as u32,
                    version: Cow::from(fcall::P92000L),
                }),
            },
        )?;

        match fcall::read(&mut r, &mut rbuf)? {
            TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Rversion(fcall::Rversion { msize, version }),
            } => {
                if version != fcall::P92000L {
                    return Err(err_other("protocol negotiation failed"));
                }
                bufsize = bufsize.min(msize as usize);
            }
            _ => return Err(err_unexpected_response()),
        }

        wbuf.truncate(bufsize);
        rbuf.truncate(bufsize);

        let fcalls = InflightFcalls::new();

        let worker_fcalls = fcalls.clone();
        let read_worker_handle = thread::spawn(move || {
            DotlClient::read_worker(r, rbuf, worker_fcalls);
        });

        Ok(DotlClient {
            state: Arc::new(DotlClientState {
                msize: bufsize.try_into().unwrap(),
                fids: Fidset::new(),
                write_state: Mutex::new(ClientWriteState { w, buf: wbuf }),
                read_worker_handle: Some(read_worker_handle),
                fcalls,
            }),
        })
    }

    fn read_worker(mut r: TcpStream, mut rbuf: Vec<u8>, fcalls: InflightFcalls) {
        loop {
            match fcall::read(&mut r, &mut rbuf) {
                Ok(response) => {
                    if let Some(resp) = fcalls.remove(response.tag) {
                        resp.send(response.fcall.clone_static()).unwrap();
                    }
                }
                Err(_) => {
                    fcalls.mark_disconnected();
                    return;
                }
            }
        }
    }

    fn fresh_fid(&self) -> Result<Fid, std::io::Error> {
        match self.state.fids.fresh() {
            Some(fid) => Ok(fid),
            None => Err(err_other("fids exhausted")),
        }
    }

    fn fcall<'a>(&self, fcall: Fcall<'a>) -> Result<Fcall<'static>, std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        {
            let mut write_state_guard = self.state.write_state.lock().unwrap();
            let write_state = write_state_guard.deref_mut();
            let w = &mut write_state.w;
            let buf = &mut write_state.buf;
            fcall::write(
                w,
                buf,
                &TaggedFcall {
                    tag: self.state.fcalls.add(tx)?,
                    fcall,
                },
            )?;
        }
        Ok(rx.recv().or_else(err_io_result)?)
    }

    pub fn attach(
        &self,
        n_uname: u32,
        uname: &str,
        aname: &str,
    ) -> Result<DotlFile, std::io::Error> {
        let fid = self.fresh_fid()?;
        match self.fcall(Fcall::Tattach(fcall::Tattach {
            afid: fcall::NOFID,
            fid: fid.id,
            n_uname,
            uname: Cow::from(uname),
            aname: Cow::from(aname),
        }))? {
            Fcall::Rattach(fcall::Rattach { qid }) => Ok(DotlFile {
                qid,
                fid,
                offset: 0,
                client: self.clone(),
            }),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}

pub struct DotlFile {
    pub qid: fcall::Qid,
    client: DotlClient,
    offset: u64,
    fid: Fid,
}

impl DotlFile {
    pub fn is_dir(&self) -> bool {
        self.qid.typ.contains(fcall::QidType::DIR)
    }

    fn primitive_walk(&self, wnames: &[&str]) -> Result<DotlFile, std::io::Error> {
        if wnames.len() > fcall::MAXWELEM {
            return Err(err_other("walk has too many wnames"));
        }

        let wnames = wnames
            .iter()
            .map(|name| Cow::from(name.to_string()))
            .collect();

        let newfid = self.client.fresh_fid()?;

        match self.client.fcall(Fcall::Twalk(fcall::Twalk {
            fid: self.fid.id,
            newfid: newfid.id,
            wnames,
        }))? {
            Fcall::Rwalk(fcall::Rwalk { wqids }) => {
                let qid = *wqids.last().unwrap_or(&self.qid);
                Ok(DotlFile {
                    fid: newfid,
                    qid,
                    client: self.client.clone(),
                    offset: 0,
                })
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn walk(&self, p: &str) -> Result<DotlFile, std::io::Error> {
        let wnames: Vec<&str> = p.split('/').filter(|x| !x.is_empty()).collect();
        if wnames.is_empty() {
            return self.primitive_walk(&wnames);
        }
        let mut f = None;
        for wnames in wnames.chunks(fcall::MAXWELEM) {
            let fref = f.as_ref().unwrap_or(self);
            f = Some(fref.primitive_walk(wnames)?);
        }
        Ok(f.unwrap())
    }

    pub fn open(&self, flags: fcall::LOpenFlags) -> Result<(), std::io::Error> {
        match self.client.fcall(Fcall::Tlopen(fcall::Tlopen {
            fid: self.fid.id,
            flags,
        }))? {
            Fcall::Rlopen(fcall::Rlopen { .. }) => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn read_dir(&self) -> Result<Vec<fcall::DirEntry<'static>>, std::io::Error> {
        if !self.is_dir() {
            return Err(err_not_dir());
        }
        let count: u32 = self.client.state.msize as u32 - fcall::READDIRHDRSZ;
        let mut offset: u64 = 0;
        let mut entries: Vec<fcall::DirEntry> = Vec::new();
        'read_all: loop {
            match self.client.fcall(Fcall::Treaddir(fcall::Treaddir {
                fid: self.fid.id,
                offset,
                count,
            }))? {
                Fcall::Rreaddir(fcall::Rreaddir { mut data }) => {
                    if data.data.is_empty() {
                        break 'read_all;
                    }
                    entries.append(&mut data.data);
                    offset = entries.last().unwrap().offset;
                }
                Fcall::Rlerror(err) => return Err(err.into_io_error()),
                _ => return Err(err_unexpected_response()),
            }
        }
        Ok(entries)
    }

    fn _close(&mut self) -> Result<(), std::io::Error> {
        match self
            .client
            .fcall(Fcall::Tclunk(fcall::Tclunk { fid: self.fid.id }))?
        {
            Fcall::Rclunk { .. } => {
                self.fid.id = fcall::NOFID;
                Ok(())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn close(mut self) -> Result<(), std::io::Error> {
        self._close()
    }
}

impl Drop for DotlFile {
    fn drop(&mut self) {
        if self.fid.id != fcall::NOFID {
            let _ = self._close();
        }
    }
}

impl Read for DotlFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let count = buf
            .len()
            .min((self.client.state.msize - fcall::IOHDRSZ) as usize) as u32;
        match self.client.fcall(Fcall::Tread(fcall::Tread {
            fid: self.fid.id,
            offset: self.offset,
            count,
        }))? {
            Fcall::Rread(fcall::Rread { data }) => {
                buf.copy_from_slice(&data[..]);
                self.offset += data.len() as u64;
                Ok(data.len())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}

impl Write for DotlFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let count = buf
            .len()
            .min((self.client.state.msize - fcall::IOHDRSZ) as usize);
        match self.client.fcall(Fcall::Twrite(fcall::Twrite {
            fid: self.fid.id,
            offset: self.offset,
            data: Cow::from(&buf[..count]),
        }))? {
            Fcall::Rwrite(fcall::Rwrite { count }) => {
                self.offset += count as u64;
                Ok(count as usize)
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self
            .client
            .fcall(Fcall::Tfsync(fcall::Tfsync { fid: self.fid.id }))?
        {
            Fcall::Rfsync { .. } => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}
