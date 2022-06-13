use super::fcall;
use super::fcall::{Fcall, TaggedFcall};
use super::transport;
use super::transport::{ReadTransport, WriteTransport};
use crossbeam_channel as channel;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::ops::DerefMut;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
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

struct Fidset {
    inner: Arc<Mutex<FidsetInner>>,
}

impl Fidset {
    fn new() -> Fidset {
        let inner = FidsetInner {
            set: HashSet::new(),
            next: fcall::NOFID,
        };
        Fidset {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    fn fresh_id(&self) -> Option<u32> {
        let mut inner = self.inner.lock().unwrap();

        if inner.set.len() == fcall::NOFID as usize {
            return None;
        }
        loop {
            if inner.next == fcall::NOFID {
                inner.next = 0;
            } else {
                inner.next += 1;
            }
            if !inner.set.contains(&inner.next) {
                let id = inner.next;
                inner.set.insert(id);
                return Some(id);
            }
        }
    }
}

struct InflightFcallsInner {
    disconnected: bool,
    map: HashMap<u16, channel::Sender<Fcall<'static>>>,
    next_tag: u16,
}

#[derive(Clone)]
struct InflightFcalls {
    inner_and_cvar: Arc<(Mutex<InflightFcallsInner>, std::sync::Condvar)>,
}

impl InflightFcalls {
    fn new() -> InflightFcalls {
        let inner = InflightFcallsInner {
            disconnected: false,
            map: HashMap::new(),
            next_tag: fcall::NOTAG,
        };
        InflightFcalls {
            inner_and_cvar: Arc::new((Mutex::new(inner), std::sync::Condvar::new())),
        }
    }

    fn mark_disconnected(&self) {
        let (ref inner, ref cvar) = self.inner_and_cvar.as_ref();
        let mut inner = inner.lock().unwrap();
        // Trigger EIO for listeners.
        inner.map.clear();
        inner.disconnected = true;
        cvar.notify_all();
    }

    fn add(&self, respond_to: channel::Sender<Fcall<'static>>) -> Result<u16, std::io::Error> {
        let (ref inner, ref cvar) = self.inner_and_cvar.as_ref();
        let mut inner = inner.lock().unwrap();

        // Use condvar to wait until there is a free tag.
        while !inner.disconnected && inner.map.len() == fcall::NOTAG as usize {
            inner = cvar.wait(inner).unwrap();
        }

        if inner.disconnected {
            return Err(err_other("disconnected"));
        };

        loop {
            if inner.next_tag == fcall::NOTAG {
                inner.next_tag = 0;
            } else {
                inner.next_tag += 1;
            }
            if !inner.map.contains_key(&inner.next_tag) {
                let tag = inner.next_tag;
                inner.map.insert(tag, respond_to);
                return Ok(tag);
            }
        }
    }

    fn remove(&self, tag: u16) -> Option<channel::Sender<Fcall<'static>>> {
        let (ref inner, ref cvar) = self.inner_and_cvar.as_ref();
        let mut inner = inner.lock().unwrap();
        cvar.notify_one();
        inner.map.remove(&tag)
    }
}

struct ClientWriteState {
    w: Box<dyn WriteTransport>,
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
        let _ = write_state.w.shutdown();
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
    pub fn over_tcp_stream(conn: TcpStream, bufsize: usize) -> Result<Client, std::io::Error> {
        let r = conn.try_clone()?;
        let w = conn;
        Client::over_transport(r, w, bufsize)
    }

    #[cfg(unix)]
    pub fn over_unix_stream(conn: UnixStream, bufsize: usize) -> Result<Client, std::io::Error> {
        let r = conn.try_clone()?;
        let w = conn;
        Client::over_transport(r, w, bufsize)
    }

    pub fn over_transport<R: ReadTransport + 'static, W: WriteTransport + 'static>(
        r: R,
        w: W,
        bufsize: usize,
    ) -> Result<Client, std::io::Error> {
        let mut r: Box<dyn ReadTransport> = std::boxed::Box::new(r);
        let mut w: Box<dyn WriteTransport> = std::boxed::Box::new(w);

        const MIN_MSIZE: u32 = 4096 + fcall::READDIRHDRSZ;
        let mut bufsize = bufsize.max(MIN_MSIZE as usize).min(u32::MAX as usize);
        let mut wbuf = Vec::with_capacity(bufsize);
        let mut rbuf = Vec::with_capacity(bufsize);

        transport::write(
            &mut w,
            &mut wbuf,
            &TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Tversion(fcall::Tversion {
                    msize: bufsize.min(u32::MAX as usize) as u32,
                    version: Cow::from("9P2000.L"),
                }),
            },
        )?;

        match transport::read(&mut r, &mut rbuf)? {
            TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Rversion(fcall::Rversion { msize, version }),
            } => {
                if version != "9P2000.L" {
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

    fn read_worker(mut r: Box<dyn ReadTransport>, mut rbuf: Vec<u8>, fcalls: InflightFcalls) {
        loop {
            match transport::read(&mut r, &mut rbuf) {
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

    fn fresh_fid(&self) -> Result<ClientFid, std::io::Error> {
        match self.state.fids.fresh_id() {
            Some(id) => Ok(ClientFid {
                client: self.clone(),
                needs_clunk: false,
                id,
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
        // Will block until a tag is free.
        let tag = self.state.fcalls.add(tx)?;
        transport::write(w, buf, &TaggedFcall { tag, fcall })?;
        drop(write_state_guard);
        rx.recv().or_else(err_io_result)
    }

    pub fn attach(
        &self,
        n_uname: u32,
        uname: &str,
        aname: &str,
    ) -> Result<(fcall::Qid, ClientFid), std::io::Error> {
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

pub struct ClientFid {
    client: Client,
    needs_clunk: bool,
    id: u32,
}

impl ClientFid {
    fn walk1(&self, wnames: &[&str]) -> Result<(Vec<fcall::Qid>, ClientFid), std::io::Error> {
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

    pub fn walk(&self, wnames: &[&str]) -> Result<(Vec<fcall::Qid>, ClientFid), std::io::Error> {
        let mut wqids = Vec::with_capacity(fcall::MAXWELEM);
        if wnames.is_empty() {
            return self.walk1(wnames);
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

    pub fn create(
        &self,
        name: &str,
        flags: fcall::LOpenFlags,
        mode: u32,
        gid: u32,
    ) -> Result<fcall::Qid, std::io::Error> {
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

    // XXX make flags atg a bitflag set?
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

    pub fn rename(&self, dir_fid: &ClientFid, name: &str) -> Result<(), std::io::Error> {
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
        new_dir_fid: &ClientFid,
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

    fn _clunk(&mut self) -> Result<(), std::io::Error> {
        if !self.needs_clunk {
            return Ok(());
        }
        self.needs_clunk = false;
        match self
            .client
            .fcall(Fcall::Tclunk(fcall::Tclunk { fid: self.id }))?
        {
            Fcall::Rclunk { .. } => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    pub fn clunk(mut self) -> Result<(), std::io::Error> {
        self._clunk()
    }

    pub fn remove(mut self) -> Result<(), std::io::Error> {
        self.needs_clunk = false;
        match self
            .client
            .fcall(Fcall::Tremove(fcall::Tremove { fid: self.id }))?
        {
            Fcall::Rremove { .. } => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}

impl Drop for ClientFid {
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
