use super::fcall;
use super::fcall::Fcall;
use crossbeam_channel as channel;
use std::borrow::Cow;
use std::boxed::Box;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;

pub struct FcallRequest {
    fcall: Fcall<'static>,
    respond: channel::Sender<Fcall<'static>>,
}

#[derive(Clone)]
pub struct DotlClient {
    fcalls: channel::Sender<FcallRequest>,
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
                self.next += 1;
                self.map.insert(tag, respond_to);
                return Some(tag);
            }
        }
    }

    fn remove(&mut self, tag: u16) -> Option<channel::Sender<Fcall<'static>>> {
        self.map.remove(&tag)
    }
}

impl DotlClient {
    pub fn connect(
        mut r: Box<dyn Read + Send>,
        mut w: Box<dyn Write + Send>,
    ) -> Result<DotlClient, std::io::Error> {
        let mut bufsize: usize = 8192;
        let mut wbuf = Vec::with_capacity(bufsize);
        let mut rbuf = Vec::with_capacity(bufsize);

        fcall::write_msg(
            &mut w,
            &mut wbuf,
            &fcall::Msg {
                tag: fcall::NOTAG,
                body: Fcall::Tversion(fcall::Tversion {
                    msize: bufsize.min(u32::MAX as usize) as u32,
                    version: Cow::from(fcall::P92000L),
                }),
            },
        )?;

        match fcall::read_msg(&mut r, &mut rbuf)? {
            fcall::Msg {
                tag: fcall::NOTAG,
                body: Fcall::Rversion(fcall::Rversion { msize, version }),
            } => {
                if version != fcall::P92000L {
                    todo!();
                }
                bufsize = bufsize.min(msize as usize);
            }
            _ => todo!(),
        }

        wbuf.truncate(bufsize);
        rbuf.truncate(bufsize);

        let (fcalls_tx, fcalls_rx) = channel::bounded(0);

        let io_worker_handle = thread::spawn(move || {
            DotlClient::io_worker(r, w, rbuf, wbuf, fcalls_rx);
        });

        Ok(DotlClient { fcalls: fcalls_tx })
    }

    fn io_worker(
        mut r: Box<dyn Read + Send>,
        mut w: Box<dyn Write + Send>,
        mut rbuf: Vec<u8>,
        mut wbuf: Vec<u8>,
        fcalls: channel::Receiver<FcallRequest>,
    ) {
        let (response_tx, response_rx) = channel::bounded(0);

        let remote_reader_worker = thread::spawn(move || loop {
            match fcall::read_msg(&mut r, &mut rbuf) {
                Ok(msg) => {
                    response_tx.send(msg.clone_static()).unwrap();
                }
                Err(err) => panic!(),
            };
        });

        let mut in_flight = InflightFcalls::new();

        loop {
            channel::select! {
                recv(response_rx) -> msg => {
                    let msg = msg.unwrap();
                    if let Some(listener) = in_flight.remove(msg.tag) {
                        let _ = listener.send(msg.body);
                    }
                },
                recv(fcalls) -> fcall => {
                    let fcall = fcall.unwrap();
                    let tag = in_flight.add(fcall.respond).unwrap();
                    fcall::write_msg(&mut w, &mut wbuf,  &fcall::Msg {
                        tag,
                        body: fcall.fcall,
                    }).unwrap();
                },
            }
        }

        remote_reader_worker.join().unwrap();
    }

    fn attach(
        &self,
        afid: u32,
        fid: u32,
        n_uname: u32,
        uname: &str,
        aname: &str,
    ) -> Result<DotlFile, std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        self.fcalls
            .send(FcallRequest {
                fcall: Fcall::Tattach(fcall::Tattach {
                    afid,
                    fid,
                    n_uname,
                    uname: Cow::from(uname.to_owned()),
                    aname: Cow::from(aname.to_owned()),
                }),
                respond: tx,
            })
            .unwrap();
        match rx.recv().unwrap() {
            Fcall::Rattach(fcall::Rattach { qid }) => Ok(DotlFile {
                qid,
                fid,
                offset: 0,
                client: self.clone(),
            }),
            _ => todo!(),
        }
    }

    fn read(&self, fid: u32, offset: u64, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let count = buf.len().min(u32::MAX as usize) as u32;
        let (tx, rx) = channel::bounded(1);
        self.fcalls
            .send(FcallRequest {
                fcall: Fcall::Tread(fcall::Tread { fid, offset, count }),
                respond: tx,
            })
            .unwrap();
        match rx.recv().unwrap() {
            Fcall::Rread(fcall::Rread { data }) => {
                buf.copy_from_slice(&data[..]);
                Ok(data.len())
            }
            _ => todo!(),
        }
    }

    fn write(&self, fid: u32, offset: u64, buf: &[u8]) -> Result<usize, std::io::Error> {
        let count = buf.len().min(u32::MAX as usize);
        let (tx, rx) = channel::bounded(1);
        self.fcalls
            .send(FcallRequest {
                fcall: Fcall::Twrite(fcall::Twrite {
                    fid,
                    offset,
                    data: Cow::from(buf[..count].to_vec()),
                }),
                respond: tx,
            })
            .unwrap();
        match rx.recv().unwrap() {
            Fcall::Rwrite(fcall::Rwrite { count }) => Ok(count as usize),
            _ => todo!(),
        }
    }

    fn fsync(&self, fid: u32) -> Result<(), std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        self.fcalls
            .send(FcallRequest {
                fcall: Fcall::Tfsync(fcall::Tfsync { fid }),
                respond: tx,
            })
            .unwrap();
        match rx.recv().unwrap() {
            Fcall::Rfsync { .. } => Ok(()),
            _ => todo!(),
        }
    }

    fn clunk(&self, fid: u32) -> Result<(), std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        self.fcalls
            .send(FcallRequest {
                fcall: Fcall::Tclunk(fcall::Tclunk { fid }),
                respond: tx,
            })
            .unwrap();
        match rx.recv().unwrap() {
            Fcall::Rclunk { .. } => Ok(()),
            _ => todo!(),
        }
    }
}

pub struct DotlFile {
    client: DotlClient,
    offset: u64,
    qid: fcall::Qid,
    fid: u32,
}

impl DotlFile {
    fn close(self) -> Result<(), std::io::Error> {
        self.client.clunk(self.fid)
    }
}

impl Drop for DotlFile {
    fn drop(&mut self) {
        let _ = self.client.clunk(self.fid);
    }
}

impl Read for DotlFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.client.read(self.fid, self.offset, buf)?;
        self.offset += n as u64;
        Ok(n)
    }
}

impl Write for DotlFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.client.write(self.fid, self.offset, buf)?;
        self.offset += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.client.fsync(self.fid)
    }
}
