use super::fcall;
use super::fcall::{Fcall, TaggedFcall};
use crossbeam_channel as channel;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread;

fn err_other(msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, msg)
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

struct FidsetInner {
    set: HashSet<u32>,
    next: u32,
}

impl FidsetInner {
    fn new() -> FidsetInner {
        FidsetInner {
            set: HashSet::new(),
            next: fcall::NOFID,
        }
    }

    fn fresh_id(&mut self) -> Option<u32> {
        if self.set.len() == (fcall::NOFID - 1) as usize {
            return None;
        }
        loop {
            if self.next == fcall::NOFID {
                self.next = 0;
            } else {
                self.next += 1;
            }
            if !self.set.contains(&self.next) {
                let id = self.next;
                self.set.insert(id);
                return Some(id);
            }
        }
    }
}

struct Fidset {
    inner: Arc<Mutex<FidsetInner>>,
}

impl Fidset {
    fn new() -> Fidset {
        Fidset {
            inner: Arc::new(Mutex::new(FidsetInner::new())),
        }
    }

    fn fresh_id(&self) -> Option<u32> {
        self.inner.lock().unwrap().fresh_id()
    }
}

struct InflightFcallsInner {
    disconnected: bool,
    map: HashMap<u16, channel::Sender<Fcall<'static>>>,
    next_tag: u16,
}

impl InflightFcallsInner {
    fn new() -> InflightFcallsInner {
        InflightFcallsInner {
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
            return Err(err_other("tags exhausted"));
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
    inner: Arc<Mutex<InflightFcallsInner>>,
}

impl InflightFcalls {
    fn new() -> InflightFcalls {
        InflightFcalls {
            inner: Arc::new(Mutex::new(InflightFcallsInner::new())),
        }
    }

    fn mark_disconnected(&self) {
        self.inner.lock().unwrap().mark_disconnected()
    }

    fn add(&self, respond_to: channel::Sender<Fcall<'static>>) -> Result<u16, std::io::Error> {
        self.inner.lock().unwrap().add(respond_to)
    }

    fn remove(&self, tag: u16) -> Option<channel::Sender<Fcall<'static>>> {
        self.inner.lock().unwrap().remove(tag)
    }
}

struct ClientWriteState {
    w: TcpStream,
    buf: Vec<u8>,
}

struct ClientState {
    msize: u32,
    fids: Fidset,
    fcalls: InflightFcalls,
    // Threads use a shared buffer and connection guarded by a mutex,
    // this slightly odd design lets us avoid copying when writing.
    write_state: Mutex<ClientWriteState>,
    read_worker_handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for ClientState {
    fn drop(&mut self) {
        let write_state = self.write_state.lock().unwrap();
        let _ = write_state.w.shutdown(std::net::Shutdown::Both);
        drop(write_state);
        if let Some(read_worker_handle) = self.read_worker_handle.take() {
            let _ = read_worker_handle.join();
        }
    }
}

#[derive(Clone)]
pub struct Client {
    state: Arc<ClientState>,
}

impl Client {
    pub fn tcp(conn: TcpStream, bufsize: usize) -> Result<Client, std::io::Error> {
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
            Client::read_worker(r, rbuf, worker_fcalls);
        });

        Ok(Client {
            state: Arc::new(ClientState {
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
        match self.state.fids.fresh_id() {
            Some(id) => Ok(Fid {
                client: self.clone(),
                needs_clunk: false,
                id: id,
            }),
            None => Err(err_other("fids exhausted")),
        }
    }

    fn fcall<'a>(&self, fcall: Fcall<'a>) -> Result<Fcall<'static>, std::io::Error> {
        let (tx, rx) = channel::bounded(1);
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
        drop(write_state_guard);
        rx.recv().or_else(err_io_result)
    }

    pub fn attach(
        &self,
        n_uname: u32,
        uname: &str,
        aname: &str,
    ) -> Result<(fcall::Qid, Fid), std::io::Error> {
        let mut fid = self.fresh_fid()?;
        match self.fcall(Fcall::Tattach(fcall::Tattach {
            afid: fcall::NOFID,
            fid: fid.id,
            n_uname,
            uname: Cow::from(uname),
            aname: Cow::from(aname),
        }))? {
            Fcall::Rattach(fcall::Rattach { qid }) => {
                fid.needs_clunk = true;
                Ok((qid, fid))
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}

pub struct Fid {
    client: Client,
    needs_clunk: bool,
    id: u32,
}

impl Fid {
    fn walk1(&self, wnames: &[&str]) -> Result<(Vec<fcall::Qid>, Fid), std::io::Error> {
        if wnames.len() > fcall::MAXWELEM {
            return Err(err_other("walk has too many wnames"));
        }

        let wnames = wnames
            .iter()
            .map(|name| Cow::from(name.to_string()))
            .collect();

        let mut new_fid = self.client.fresh_fid()?;

        match self.client.fcall(Fcall::Twalk(fcall::Twalk {
            fid: self.id,
            new_fid: new_fid.id,
            wnames,
        }))? {
            Fcall::Rwalk(fcall::Rwalk { wqids }) => {
                new_fid.needs_clunk = true;
                Ok((wqids, new_fid))
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn walk(&self, wnames: &[&str]) -> Result<(Vec<fcall::Qid>, Fid), std::io::Error> {
        let mut wqids = Vec::with_capacity(fcall::MAXWELEM);
        if wnames.is_empty() {
            return self.walk1(&wnames);
        }
        let mut f = None;
        for wnames in wnames.chunks(fcall::MAXWELEM) {
            let fref = f.as_ref().unwrap_or(self);
            let (mut new_wqids, new_f) = fref.walk1(wnames)?;
            wqids.append(&mut new_wqids);
            f = Some(new_f);
        }
        Ok((wqids, f.unwrap()))
    }

    pub fn open(&self, flags: fcall::LOpenFlags) -> Result<fcall::Qid, std::io::Error> {
        match self.client.fcall(Fcall::Tlopen(fcall::Tlopen {
            fid: self.id,
            flags,
        }))? {
            Fcall::Rlopen(fcall::Rlopen { qid, .. }) => Ok(qid),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn create(&self, name: &str, flags: fcall::LOpenFlags, mode: u32, gid: u32) -> Result<fcall::Qid, std::io::Error> {
        match self.client.fcall(Fcall::Tlcreate(fcall::Tlcreate {
            fid: self.id,
            flags,
            mode,
            gid,
            name: Cow::from(name),
        }))? {
            Fcall::Rlcreate(fcall::Rlcreate { qid, .. }) => Ok(qid),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn read_dir1(&self, offset: u64) -> Result<Vec<fcall::DirEntry<'static>>, std::io::Error> {
        let count: u32 = self.client.state.msize as u32 - fcall::READDIRHDRSZ;
        match self.client.fcall(Fcall::Treaddir(fcall::Treaddir {
            fid: self.id,
            offset,
            count,
        }))? {
            Fcall::Rreaddir(fcall::Rreaddir { data }) => Ok(data.data),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn read_dir(&self) -> Result<Vec<fcall::DirEntry<'static>>, std::io::Error> {
        let mut offset: u64 = 0;
        let mut entries: Vec<fcall::DirEntry> = Vec::new();
        'read_all: loop {
            let mut new_entries = self.read_dir1(offset)?;
            if new_entries.is_empty() {
                break 'read_all;
            }
            entries.append(&mut new_entries);
            offset = entries.last().unwrap().offset;
        }
        Ok(entries)
    }

    pub fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let count = buf
            .len()
            .min((self.client.state.msize - fcall::IOHDRSZ) as usize) as u32;
        match self.client.fcall(Fcall::Tread(fcall::Tread {
            fid: self.id,
            offset,
            count,
        }))? {
            Fcall::Rread(fcall::Rread { data }) => {
                if !data.is_empty() {
                    let dest = &mut buf[..data.len()];
                    dest.copy_from_slice(&data);
                }
                Ok(data.len())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn write(&self, offset: u64, buf: &[u8]) -> Result<usize, std::io::Error> {
        let count = buf
            .len()
            .min((self.client.state.msize - fcall::IOHDRSZ) as usize);
        match self.client.fcall(Fcall::Twrite(fcall::Twrite {
            fid: self.id,
            offset,
            data: Cow::from(&buf[..count]),
        }))? {
            Fcall::Rwrite(fcall::Rwrite { count }) => Ok(count as usize),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn mkdir(&self, name: &str, mode: u32, gid: u32) -> Result<fcall::Qid, std::io::Error> {
        match self.client.fcall(Fcall::Tmkdir(fcall::Tmkdir {
            dfid: self.id,
            name: Cow::from(name),
            mode,
            gid,
        }))? {
            Fcall::Rmkdir(fcall::Rmkdir { qid }) => Ok(qid),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    // XXX make flags an bigflag set?
    pub fn unlinkat(&self, name: &str, flags: u32) -> Result<(), std::io::Error> {
        match self.client.fcall(Fcall::Tunlinkat(fcall::Tunlinkat {
            dfid: self.id,
            name: Cow::from(name),
            flags,
        }))? {
            Fcall::Runlinkat(fcall::Runlinkat { .. }) => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn rename(&self, dir_fid: &Fid, name: &str) -> Result<(), std::io::Error> {
        match self.client.fcall(Fcall::Trename(fcall::Trename {
            fid: self.id,
            dfid: dir_fid.id,
            name: Cow::from(name),
        }))? {
            Fcall::Rrename(fcall::Rrename { .. }) => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn renameat(
        &self,
        oldname: &str,
        new_dir_fid: &Fid,
        newname: &str,
    ) -> Result<(), std::io::Error> {
        match self.client.fcall(Fcall::Trenameat(fcall::Trenameat {
            olddfid: self.id,
            newdfid: new_dir_fid.id,
            oldname: Cow::from(oldname),
            newname: Cow::from(newname),
        }))? {
            Fcall::Rrenameat(fcall::Rrenameat { .. }) => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn getattr(&self, mask: fcall::GetattrMask) -> Result<fcall::Rgetattr, std::io::Error> {
        match self.client.fcall(Fcall::Tgetattr(fcall::Tgetattr {
            fid: self.id,
            req_mask: mask,
        }))? {
            Fcall::Rgetattr(resp) => Ok(resp),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn setattr(
        &self,
        valid: fcall::SetattrMask,
        stat: fcall::SetAttr,
    ) -> Result<(), std::io::Error> {
        match self.client.fcall(Fcall::Tsetattr(fcall::Tsetattr {
            fid: self.id,
            valid,
            stat,
        }))? {
            Fcall::Rsetattr(fcall::Rsetattr { .. }) => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn fsync(&self) -> Result<(), std::io::Error> {
        match self
            .client
            .fcall(Fcall::Tfsync(fcall::Tfsync { fid: self.id }))?
        {
            Fcall::Rfsync { .. } => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    fn _clunk(&mut self) -> Result<(), std::io::Error> {
        if !self.needs_clunk {
            return Ok(());
        }
        match self
            .client
            .fcall(Fcall::Tclunk(fcall::Tclunk { fid: self.id }))?
        {
            Fcall::Rclunk { .. } => {
                self.needs_clunk = false;
                Ok(())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn lock(&mut self, flock: fcall::Flock) -> Result<fcall::LockStatus, std::io::Error> {
        match self.client.fcall(Fcall::Tlock(fcall::Tlock {
            fid: self.id,
            flock,
        }))? {
            Fcall::Rlock(fcall::Rlock { status }) => Ok(status),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn clunk(mut self) -> Result<(), std::io::Error> {
        self._clunk()
    }

    pub fn remove(mut self) -> Result<(), std::io::Error> {
        match self
            .client
            .fcall(Fcall::Tremove(fcall::Tremove { fid: self.id }))?
        {
            Fcall::Rremove { .. } => {
                self.needs_clunk = false;
                Ok(())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}

impl Drop for Fid {
    fn drop(&mut self) {
        let _ = self._clunk();
        self.client
            .state
            .fids
            .inner
            .lock()
            .unwrap()
            .set
            .remove(&self.id);
    }
}
