use super::fcall;
use super::fcall::{Fcall, TaggedFcall};
use crossbeam_channel as channel;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

// Wrapper around fid that automatically removes itself from the active set
// when dropped.
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

struct DotlClientState {
    msize: u32,
    fids: Fidset,
    read_worker_handle: Option<std::thread::JoinHandle<()>>,
    dispatch_worker_handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for DotlClientState {
    fn drop(&mut self) {
        if let Some(dispatch_worker_handle) = self.dispatch_worker_handle.take() {
            let _ = dispatch_worker_handle.join();
        }
        if let Some(read_worker_handle) = self.read_worker_handle.take() {
            let _ = read_worker_handle.join();
        }
    }
}

#[derive(Debug)]
pub struct FcallRequest {
    fcall: Fcall<'static>,
    respond: channel::Sender<Fcall<'static>>,
}

struct InflightFcalls {
    map: HashMap<u16, channel::Sender<Fcall<'static>>>,
    next: u16,
}

impl InflightFcalls {
    fn new() -> InflightFcalls {
        InflightFcalls {
            map: HashMap::new(),
            next: fcall::NOTAG,
        }
    }

    fn add(&mut self, respond_to: channel::Sender<Fcall<'static>>) -> Option<u16> {
        if self.map.len() == (fcall::NOTAG - 1) as usize {
            return None;
        }
        loop {
            if self.next == fcall::NOTAG {
                self.next = 0;
            } else {
                self.next += 1;
            }
            if !self.map.contains_key(&self.next) {
                let tag = self.next;
                self.map.insert(tag, respond_to);
                return Some(tag);
            }
        }
    }

    fn remove(&mut self, tag: u16) -> Option<channel::Sender<Fcall<'static>>> {
        self.map.remove(&tag)
    }
}

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

#[derive(Clone)]
pub struct DotlClient {
    // This is declared before state so that it
    // is dropped first, thus letting threads exit.
    requests: channel::Sender<FcallRequest>,
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

        let (requests_tx, requests_rx) = channel::bounded(0);
        let (responses_tx, responses_rx) = channel::bounded(0);

        let read_worker_handle = thread::spawn(move || {
            DotlClient::read_worker(r, rbuf, responses_tx);
        });

        let dispatch_worker_handle = thread::spawn(move || {
            DotlClient::dispatch_worker(w, wbuf, requests_rx, responses_rx);
        });

        Ok(DotlClient {
            state: Arc::new(DotlClientState {
                msize: bufsize.try_into().unwrap(),
                fids: Fidset::new(),
                dispatch_worker_handle: Some(dispatch_worker_handle),
                read_worker_handle: Some(read_worker_handle),
            }),
            requests: requests_tx,
        })
    }

    fn read_worker(
        mut r: TcpStream,
        mut rbuf: Vec<u8>,
        responses: channel::Sender<TaggedFcall<'static>>,
    ) {
        loop {
            match fcall::read(&mut r, &mut rbuf) {
                Ok(msg) => {
                    if responses.send(msg.clone_static()).is_err() {
                        return;
                    };
                }
                Err(_) => return,
            }
        }
    }

    fn dispatch_worker(
        mut w: TcpStream,
        mut wbuf: Vec<u8>,
        requests: channel::Receiver<FcallRequest>,
        responses: channel::Receiver<TaggedFcall<'static>>,
    ) {
        let mut in_flight = InflightFcalls::new();

        'events: loop {
            channel::select! {
                recv(responses) -> response => {
                    match response {
                        Ok(response) => {
                            if let Some(respond_to) = in_flight.remove(response.tag) {
                                let _ = respond_to.send(response.fcall);
                            }
                        }
                        Err(_) => break 'events,
                    }
                },
                recv(requests) -> request => {
                    match request {
                        Ok(request) => if let Some(tag) = in_flight.add(request.respond) {
                            if fcall::write(&mut w, &mut wbuf,  &TaggedFcall {
                                tag,
                                fcall: request.fcall,
                            }).is_err() {
                                break 'events;
                            };
                        } else {
                            // No tags left, triggers an EIO by dropping channel.
                        }
                        Err(_) => break 'events,
                    }
                },
            }
        }

        // Cancel in flight requests immediately.
        drop(in_flight);
        // Ensure io worker will exit.
        drop(responses);
        // Disconnect from remote.
        let _ = w.shutdown(std::net::Shutdown::Write);
    }

    fn fresh_fid(&self) -> Result<Fid, std::io::Error> {
        match self.state.fids.fresh() {
            Some(fid) => Ok(fid),
            None => Err(err_other("fids exhausted")),
        }
    }

    fn fcall<'a>(&self, fcall: &Fcall<'a>) -> Result<Fcall<'static>, std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        // XXX If we just write directly from this thread
        // and use a mutex this copy will not be needed.
        self.requests
            .send(FcallRequest {
                fcall: fcall.clone_static(),
                respond: tx,
            })
            .or_else(err_io_result)?;
        Ok(rx.recv().or_else(err_io_result)?)
    }

    pub fn attach(
        &self,
        n_uname: u32,
        uname: &str,
        aname: &str,
    ) -> Result<DotlFile, std::io::Error> {
        let fid = self.fresh_fid()?;
        match self.fcall(&Fcall::Tattach(fcall::Tattach {
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

        match self.client.fcall(&Fcall::Twalk(fcall::Twalk {
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
        match self.client.fcall(&Fcall::Tlopen(fcall::Tlopen {
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
            match self.client.fcall(&Fcall::Treaddir(fcall::Treaddir {
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

    pub fn close(&mut self) -> Result<(), std::io::Error> {
        match self
            .client
            .fcall(&Fcall::Tclunk(fcall::Tclunk { fid: self.fid.id }))?
        {
            Fcall::Rclunk { .. } => {
                self.fid.id = fcall::NOFID;
                Ok(())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}

impl Drop for DotlFile {
    fn drop(&mut self) {
        if self.fid.id != fcall::NOFID {
            let _ = self.close();
        }
    }
}

impl Read for DotlFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let count = buf
            .len()
            .min((self.client.state.msize - fcall::IOHDRSZ) as usize) as u32;
        match self.client.fcall(&Fcall::Tread(fcall::Tread {
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
        match self.client.fcall(&Fcall::Twrite(fcall::Twrite {
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
            .fcall(&Fcall::Tfsync(fcall::Tfsync { fid: self.fid.id }))?
        {
            Fcall::Rfsync { .. } => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}
