use super::fcall;
use super::fcall::{Fcall, TaggedFcall};
use crossbeam_channel as channel;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

// Wrapper around fid that automatically removes itself from the active set.
// Note, it does *not* clunk itself, 'File' serves that purpose.
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
    next: u32,
}

impl Fidset {
    fn new() -> Fidset {
        Fidset {
            set: Arc::new(Mutex::new(HashSet::new())),
            next: fcall::NOFID,
        }
    }

    fn fresh(&mut self) -> Option<Fid> {
        let mut set = self.set.lock().unwrap();
        if set.len() == (fcall::NOFID - 1) as usize {
            return None;
        }
        loop {
            if self.next == fcall::NOFID {
                self.next = 0;
            } else {
                self.next += 1;
            }
            if !set.contains(&self.next) {
                let id = self.next;
                set.insert(id);
                return Some(Fid{ id, set: self.set.clone() });
            }
        }
    }

}

struct DotlClientState {
    read_worker_handle: Option<std::thread::JoinHandle<()>>,
    dispatch_worker_handle: Option<std::thread::JoinHandle<()>>,
    fids: Fidset,
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

fn eio() -> std::io::Error {
    err_other("io error")
}

fn eio_result<R, E>(_e: E) -> Result<R, std::io::Error> {
    Err(eio())
}

fn err_unexpected_response() -> std::io::Error {
    err_other("unexpected response from server")
}

#[derive(Clone)]
pub struct DotlClient {
    // This is declared before state so that it
    // is dropped first, thus letting threads exit.
    requests: channel::Sender<FcallRequest>,
    state: Arc<Mutex<DotlClientState>>,
}

impl DotlClient {
    pub fn tcp(conn: TcpStream, bufsize: usize) -> Result<DotlClient, std::io::Error> {
        let mut r = conn;
        let mut w = r.try_clone()?;

        let mut bufsize = bufsize.max(512);
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
            state: Arc::new(Mutex::new(DotlClientState {
                dispatch_worker_handle: Some(dispatch_worker_handle),
                read_worker_handle: Some(read_worker_handle),
                fids: Fidset::new(),
            })),
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

    fn fresh_fid(&self) -> Option<Fid> {
        self.state.lock().unwrap().fids.fresh()
    }

    pub fn attach(
        &self,
        n_uname: u32,
        uname: &str,
        aname: &str,
    ) -> Result<DotlFile, std::io::Error> {
        let (tx, rx) = channel::bounded(1);

        let fid = match self.fresh_fid() {
            Some(fid) => fid,
            None => return Err(eio()),
        };

        self.requests
            .send(FcallRequest {
                fcall: Fcall::Tattach(fcall::Tattach {
                    afid: fcall::NOFID,
                    fid: fid.id,
                    n_uname,
                    uname: Cow::from(uname.to_owned()),
                    aname: Cow::from(aname.to_owned()),
                }),
                respond: tx,
            })
            .or_else(eio_result)?;

        match rx.recv().or_else(eio_result)? {
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

    fn walk(&self, fid: &Fid, p: &str) -> Result<DotlFile, std::io::Error> {

        let wnames = p.split("/").filter(|x| !x.is_empty()).map(|name| Cow::from(name.to_string())).collect();

        let newfid = match self.fresh_fid() {
            Some(fid) => fid,
            None => return Err(eio()),
        };

        let (tx, rx) = channel::bounded(1);
        self.requests
            .send(FcallRequest {
                fcall: Fcall::Twalk(fcall::Twalk { fid: fid.id, newfid: newfid.id, wnames }),
                respond: tx,
            })
            .or_else(eio_result)?;
        
        match rx.recv().or_else(eio_result)? {
            Fcall::Rwalk(fcall::Rwalk { wqids }) => {
                let qid = if let Some(qid) = wqids.last() {
                    *qid
                } else {
                    // Overwritten by caller if necessary.
                    fcall::Qid{typ: fcall::QidType::FILE, path: 0, version: 0}
                };
                Ok(DotlFile {
                    qid,
                    fid: newfid,
                    offset: 0,
                    client: self.clone(),
                })
            } ,
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    fn open(&self, fid: &Fid, flags: u32) -> Result<(), std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        self.requests
            .send(FcallRequest {
                fcall: Fcall::Tlopen(fcall::Tlopen { fid: fid.id, flags }),
                respond: tx,
            })
            .or_else(eio_result)?;
        match rx.recv().or_else(eio_result)? {
            Fcall::Rlopen(fcall::Rlopen { .. }) => {
                Ok(())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    fn read_dir(&self, fid: &Fid) -> Result<Vec<fcall::DirEntry>, std::io::Error> {
        let count: u32 = 8192; // XXX msize - IOHEADR ?
        let mut offset: u64 = 0;
        let mut entries: Vec<fcall::DirEntry> = Vec::new();
        'read_all: loop {
            let (tx, rx) = channel::bounded(1);
            self.requests
                .send(FcallRequest {
                    fcall: Fcall::Treaddir(fcall::Treaddir { fid: fid.id, offset, count }),
                    respond: tx,
                })
                .or_else(eio_result)?;
            match rx.recv().or_else(eio_result)? {
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
            dbg!("YYY");
        }
        Ok(entries)
    }

    fn read(&self, fid: &Fid, offset: u64, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let count = buf.len().min(u32::MAX as usize) as u32;
        let (tx, rx) = channel::bounded(1);
        self.requests
            .send(FcallRequest {
                fcall: Fcall::Tread(fcall::Tread { fid: fid.id, offset, count }),
                respond: tx,
            })
            .or_else(eio_result)?;
        match rx.recv().or_else(eio_result)? {
            Fcall::Rread(fcall::Rread { data }) => {
                buf.copy_from_slice(&data[..]);
                Ok(data.len())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    fn write(&self, fid: &Fid, offset: u64, buf: &[u8]) -> Result<usize, std::io::Error> {
        let count = buf.len().min(u32::MAX as usize);
        let (tx, rx) = channel::bounded(1);
        self.requests
            .send(FcallRequest {
                fcall: Fcall::Twrite(fcall::Twrite {
                    fid: fid.id,
                    offset,
                    data: Cow::from(buf[..count].to_vec()),
                }),
                respond: tx,
            })
            .or_else(eio_result)?;
        match rx.recv().or_else(eio_result)? {
            Fcall::Rwrite(fcall::Rwrite { count }) => Ok(count as usize),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    fn fsync(&self, fid: &Fid) -> Result<(), std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        self.requests
            .send(FcallRequest {
                fcall: Fcall::Tfsync(fcall::Tfsync { fid: fid.id }),
                respond: tx,
            })
            .or_else(eio_result)?;
        match rx.recv().or_else(eio_result)? {
            Fcall::Rfsync { .. } => Ok(()),
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }

    fn clunk(&self, fid: &Fid) -> Result<(), std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        self.requests
            .send(FcallRequest {
                fcall: Fcall::Tclunk(fcall::Tclunk { fid: fid.id }),
                respond: tx,
            })
            .or_else(eio_result)?;
        match rx.recv().or_else(eio_result)? {
            Fcall::Rclunk { .. } => {
                Ok(())
            }
            Fcall::Rlerror(err) => Err(err.into_io_error()),
            _ => Err(err_unexpected_response()),
        }
    }
}

pub struct DotlFile {
    client: DotlClient,
    qid: fcall::Qid,
    offset: u64,
    fid: Fid,
}

impl DotlFile {

    pub fn open(&self, flags: u32) -> Result<(), std::io::Error> {
         self.client.open(&self.fid, flags)
    }
    
    pub fn walk(&self, p: &str) -> Result<DotlFile, std::io::Error> {
        let mut f = self.client.walk(&self.fid, p)?;
        if p.is_empty() {
            // XXX: Is this special case a problem in our factoring?
            f.qid = self.qid;
        }
        Ok(f)
    }

    pub fn read_dir(&self) -> Result<Vec<fcall::DirEntry>, std::io::Error> {
        self.client.read_dir(&self.fid)
    }

    pub fn close(mut self) -> Result<(), std::io::Error> {
        self.client.clunk(&self.fid)?;
        self.fid.id = fcall::NOFID;
        Ok(())
    }
}

impl Drop for DotlFile {
    fn drop(&mut self) {
        if self.fid.id != fcall::NOFID {
            let _ = self.client.clunk(&self.fid);
        }
    }
}

impl Read for DotlFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.client.read(&self.fid, self.offset, buf)?;
        self.offset += n as u64;
        Ok(n)
    }
}

impl Write for DotlFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.client.write(&self.fid, self.offset, buf)?;
        self.offset += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.client.fsync(&self.fid)
    }
}
