use super::fcall;
use crossbeam_channel as channel;
use std::boxed::Box;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Debug)]
enum Request {
    Read {
        read : fcall::Tread,
        // XXX Do with no copy?
        respond: channel::Sender<Result<fcall::Rread, std::io::Error>>,
    },
    Clunk {
        fid: u32,
        respond: channel::Sender<Result<(), std::io::Error>>,
    },
}

#[derive(Clone)]
pub struct DotlClient {
    tx: channel::Sender<Request>,
}

pub struct DotlFile {
    client: DotlClient,
    offset: u64,
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

impl DotlClient {
    pub fn new(_r: Box<dyn Read + Send>, _w: Box<dyn Write>) -> DotlClient {
        let (tx, _rx) = channel::bounded(0);
        DotlClient { tx }
    }

    fn io_worker(
        reqs: channel::Receiver<Request>,
        mut r: Box<dyn Read + Send>,
        mut w: Box<dyn Write>,
    ) {
        let (server_tx, server_rx) = channel::bounded(0);

        let remote_reader_worker = thread::spawn(move || {
            let mut buf = Vec::with_capacity(1024 * 1024); // XXX Should be negotiated msize.
            loop {
                match fcall::read_msg(&mut r, &mut buf) {
                    Ok(msg) => {
                        server_tx.send(msg.clone_static()).unwrap();
                    }
                    Err(err) => panic!(),
                };
            }
        });

        loop {
            channel::select! {
                recv(server_rx) -> msg => {
                    todo!();
                },
                recv(reqs) -> req => {
                    todo!();
                },
            }
        }

        remote_reader_worker.join().unwrap();
    }

    fn read(&self, fid: u32, offset: u64, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let amount = buf.len().min(u32::MAX as usize) as u32;
        let (tx, rx) = channel::bounded(1);
        self.tx
            .send(Request::Read {
                read: fcall::Tread {
                	fid,
                	offset,
                	amount,
                }
                respond: tx,
            })
            .unwrap();
        match rx.recv().unwrap() {
            Ok(fcall::Fcall::Rread{
            	..
            }) => {
            	panic!(),
            }
            Ok(_) => panic!(),
            Err(_err) => panic!(),
        }
    }

    fn clunk(&self, fid: u32) -> Result<(), std::io::Error> {
        let (tx, rx) = channel::bounded(1);
        self.tx.send(Request::Clunk { fid, respond: tx }).unwrap();
        match rx.recv().unwrap() {
            Ok(()) => Ok(()),
            Err(_err) => panic!(),
        }
    }
}
