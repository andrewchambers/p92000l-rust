use super::errno;
use super::fcall;
use super::fcall::*;
use std::borrow::Cow;
use std::boxed::Box;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

struct WriteState {
    buf: Vec<u8>,
    conn: std::net::TcpStream,
}

#[derive(Clone)]
pub struct FcallResponse {
    pub tag: u16,
    wstate: Arc<Mutex<WriteState>>,
}

impl<'a> FcallResponse {
    fn _send(&mut self, resp: Fcall<'_>) {
        let mut wstate = self.wstate.lock().unwrap();
        let wstate = wstate.deref_mut();
        let _ = fcall::write(
            &mut wstate.conn,
            &mut wstate.buf,
            &fcall::TaggedFcall {
                tag: self.tag,
                fcall: resp,
            },
        );
        self.tag = fcall::NOTAG;
    }

    pub fn send<R: Into<Fcall<'a>>>(mut self, r: R) {
        self._send(r.into())
    }
}

impl Drop for FcallResponse {
    fn drop(&mut self) {
        if self.tag != fcall::NOTAG {
            self._send(Rlerror { ecode: errno::EIO }.into())
        }
    }
}

pub trait Filesystem {
    fn statfs(&mut self, _req: &Tstatfs, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn lopen(&mut self, _req: &Tlopen, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn lcreate(&mut self, _req: &Tlcreate, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn symlink(&mut self, _req: &Tsymlink, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn mknod(&mut self, _req: &Tmknod, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn rename(&mut self, _req: &Trename, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn readlink(&mut self, _req: &Treadlink, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn getattr(&mut self, _req: &Tgetattr, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn setattr(&mut self, _req: &Tsetattr, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn xattrwalk(&mut self, _req: &Txattrwalk, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn xattrcreate(&mut self, _req: &Txattrcreate, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn readdir(&mut self, _req: &Treaddir, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn fsync(&mut self, _req: &Tfsync, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn lock(&mut self, _req: &Tlock, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn getlock(&mut self, _req: &Tgetlock, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn link(&mut self, _req: &Tlink, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn mkdir(&mut self, _req: &Tmkdir, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn renameat(&mut self, _req: &Trenameat, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn unlinkat(&mut self, _req: &Tunlinkat, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn auth(&mut self, _req: &Tauth, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn attach(&mut self, _req: &Tattach, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn flush(&mut self, _req: &Tflush, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn walk(&mut self, _req: &Twalk, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn read(&mut self, _req: &Tread, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn write(&mut self, _req: &Twrite, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn clunk(&mut self, _req: &Tclunk, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn remove(&mut self, _req: &Tremove, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }
}

pub trait ThreadedFilesystem {
    fn statfs(&self, _req: &Tstatfs, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn lopen(&self, _req: &Tlopen, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn lcreate(&self, _req: &Tlcreate, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn symlink(&self, _req: &Tsymlink, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn mknod(&self, _req: &Tmknod, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn rename(&self, _req: &Trename, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn readlink(&self, _req: &Treadlink, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn getattr(&self, _req: &Tgetattr, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn setattr(&self, _req: &Tsetattr, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn xattrwalk(&self, _req: &Txattrwalk, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn xattrcreate(&self, _req: &Txattrcreate, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn readdir(&self, _req: &Treaddir, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn fsync(&self, _req: &Tfsync, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn lock(&self, _req: &Tlock, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn getlock(&self, _req: &Tgetlock, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn link(&self, _req: &Tlink, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn mkdir(&self, _req: &Tmkdir, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn renameat(&self, _req: &Trenameat, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn unlinkat(&self, _req: &Tunlinkat, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn auth(&self, _req: &Tauth, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn attach(&self, _req: &Tattach, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn flush(&self, _req: &Tflush, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn walk(&self, _req: &Twalk, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn read(&self, _req: &Tread, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn write(&self, _req: &Twrite, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn clunk(&self, _req: &Tclunk, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }

    fn remove(&self, _req: &Tremove, resp: FcallResponse) {
        resp.send(Rlerror {
            ecode: errno::EOPNOTSUPP,
        })
    }
}

pub struct ThreadPoolServer<Fs: 'static + ThreadedFilesystem + Send + Sync> {
    fs: Arc<Fs>,
    workers: Vec<std::thread::JoinHandle<()>>,
    dispatch_tx: crossbeam_channel::Sender<Box<dyn Send + FnOnce()>>,
}

impl<Fs: 'static + ThreadedFilesystem + Send + Sync> ThreadPoolServer<Fs> {
    pub fn new(fs: Fs) -> ThreadPoolServer<Fs> {
        let mut workers = Vec::new();

        let (dispatch_tx, dispatch_rx): (crossbeam_channel::Sender<Box<dyn Send + FnOnce()>>, _) =
            crossbeam_channel::bounded(0);

        const N_WORKERS: usize = 8;
        for _i in 0..N_WORKERS {
            let rx = dispatch_rx.clone();
            workers.push(std::thread::spawn(move || loop {
                match rx.recv() {
                    Ok(f) => f(),
                    _ => todo!(),
                }
            }))
        }

        Self {
            fs: Arc::new(fs),
            workers,
            dispatch_tx,
        }
    }

    fn dispatch(&mut self, f: Box<dyn Send + FnOnce()>) {
        let _ = self.dispatch_tx.send(f);
    }
}

impl<Fs: 'static + ThreadedFilesystem + Send + Sync> Drop for ThreadPoolServer<Fs> {
    fn drop(&mut self) {
        // Close sender by overwriting it.
        let (dispatch_tx, _) = crossbeam_channel::bounded(0);
        self.dispatch_tx = dispatch_tx;

        for w in self.workers.drain(..) {
            w.join().unwrap()
        }
    }
}

impl<Fs: 'static + ThreadedFilesystem + Send + Sync> Filesystem for ThreadPoolServer<Fs> {
    fn statfs(&mut self, req: &Tstatfs, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.statfs(&req, resp)));
    }

    fn lopen(&mut self, req: &Tlopen, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.lopen(&req, resp)));
    }

    fn lcreate(&mut self, req: &Tlcreate, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.lcreate(&req, resp)));
    }

    fn symlink(&mut self, req: &Tsymlink, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.symlink(&req, resp)));
    }

    fn mknod(&mut self, req: &Tmknod, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.mknod(&req, resp)));
    }

    fn rename(&mut self, req: &Trename, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.rename(&req, resp)));
    }

    fn readlink(&mut self, req: &Treadlink, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.readlink(&req, resp)));
    }

    fn getattr(&mut self, req: &Tgetattr, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.getattr(&req, resp)));
    }

    fn setattr(&mut self, req: &Tsetattr, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.setattr(&req, resp)));
    }

    fn xattrwalk(&mut self, req: &Txattrwalk, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.xattrwalk(&req, resp)));
    }

    fn xattrcreate(&mut self, req: &Txattrcreate, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.xattrcreate(&req, resp)));
    }

    fn readdir(&mut self, req: &Treaddir, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.readdir(&req, resp)));
    }

    fn fsync(&mut self, req: &Tfsync, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.fsync(&req, resp)));
    }

    fn lock(&mut self, req: &Tlock, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.lock(&req, resp)));
    }

    fn getlock(&mut self, req: &Tgetlock, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.getlock(&req, resp)));
    }

    fn link(&mut self, req: &Tlink, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.link(&req, resp)));
    }

    fn mkdir(&mut self, req: &Tmkdir, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.mkdir(&req, resp)));
    }

    fn renameat(&mut self, req: &Trenameat, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.renameat(&req, resp)));
    }

    fn unlinkat(&mut self, req: &Tunlinkat, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.unlinkat(&req, resp)));
    }

    fn auth(&mut self, req: &Tauth, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.auth(&req, resp)));
    }

    fn attach(&mut self, req: &Tattach, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.attach(&req, resp)));
    }

    fn flush(&mut self, req: &Tflush, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.flush(&req, resp)));
    }

    fn walk(&mut self, req: &Twalk, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.walk(&req, resp)));
    }

    fn read(&mut self, req: &Tread, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.read(&req, resp)));
    }

    fn write(&mut self, req: &Twrite, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone_static();
        self.dispatch(Box::new(move || fs.write(&req, resp)));
    }

    fn clunk(&mut self, req: &Tclunk, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.clunk(&req, resp)));
    }

    fn remove(&mut self, req: &Tremove, resp: FcallResponse) {
        let fs = self.fs.clone();
        let req = req.clone();
        self.dispatch(Box::new(move || fs.remove(&req, resp)));
    }
}

pub fn serve<F>(mut conn: std::net::TcpStream, fs: &mut F, mut bufsize: usize)
where
    F: Filesystem,
{
    bufsize = bufsize
        .min(u32::MAX as usize)
        .max(4096 + fcall::READDIRHDRSZ as usize);

    let mut rbuf: Vec<u8> = Vec::with_capacity(bufsize);
    let mut wbuf: Vec<u8> = Vec::with_capacity(bufsize);

    // Handle version and size buffer.
    match fcall::read(&mut conn, &mut rbuf) {
        Ok(fcall::TaggedFcall {
            tag: fcall::NOTAG,
            fcall:
                Fcall::Tversion(Tversion {
                    ref msize,
                    ref version,
                }),
        }) => {
            let msize = (*msize).min(bufsize as u32);

            let rversion = if version == "9P2000.L" {
                Rversion {
                    version: version.clone(),
                    msize,
                }
            } else {
                Rversion {
                    version: Cow::from("unknown"),
                    msize,
                }
            };

            if fcall::write(
                &mut conn,
                &mut wbuf,
                &fcall::TaggedFcall {
                    tag: fcall::NOTAG,
                    fcall: rversion.into(),
                },
            )
            .is_err()
            {
                return;
            }

            bufsize = msize as usize;
        }
        _ => return,
    }

    rbuf.resize(bufsize, 0);
    wbuf.resize(bufsize, 0);

    let (wconn, mut rconn) = if let Ok(wconn) = conn.try_clone() {
        (wconn, conn)
    } else {
        return;
    };

    let wstate = Arc::new(Mutex::new(WriteState {
        conn: wconn,
        buf: wbuf,
    }));

    loop {
        let (fcall, resp) = match fcall::read(&mut rconn, &mut rbuf) {
            Ok(fcall::TaggedFcall { tag, fcall }) => (
                fcall,
                FcallResponse {
                    tag,
                    wstate: wstate.clone(),
                },
            ),
            _ => return,
        };

        match fcall {
            Fcall::Tstatfs(req) => fs.statfs(&req, resp),
            Fcall::Tlopen(req) => fs.lopen(&req, resp),
            Fcall::Tlcreate(req) => fs.lcreate(&req, resp),
            Fcall::Tsymlink(req) => fs.symlink(&req, resp),
            Fcall::Tmknod(req) => fs.mknod(&req, resp),
            Fcall::Treadlink(req) => fs.readlink(&req, resp),
            Fcall::Tgetattr(req) => fs.getattr(&req, resp),
            Fcall::Tsetattr(req) => fs.setattr(&req, resp),
            Fcall::Treaddir(req) => fs.readdir(&req, resp),
            Fcall::Tfsync(req) => fs.fsync(&req, resp),
            Fcall::Tmkdir(req) => fs.mkdir(&req, resp),
            Fcall::Tflush(req) => fs.flush(&req, resp),
            Fcall::Tread(req) => fs.read(&req, resp),
            Fcall::Twrite(req) => fs.write(&req, resp),
            Fcall::Tclunk(req) => fs.clunk(&req, resp),
            Fcall::Tremove(req) => fs.remove(&req, resp),
            Fcall::Trename(req) => fs.rename(&req, resp),
            Fcall::Tlink(req) => fs.link(&req, resp),
            Fcall::Trenameat(req) => fs.renameat(&req, resp),
            Fcall::Tunlinkat(req) => fs.unlinkat(&req, resp),
            Fcall::Tlock(req) => fs.lock(&req, resp),
            Fcall::Tgetlock(req) => fs.getlock(&req, resp),
            Fcall::Tauth(req) => fs.auth(&req, resp),
            Fcall::Tattach(req) => fs.attach(&req, resp),
            Fcall::Twalk(req) => fs.walk(&req, resp),
            Fcall::Txattrwalk(req) => fs.xattrwalk(&req, resp),
            Fcall::Txattrcreate(req) => fs.xattrcreate(&req, resp),
            _ => return,
        };
    }
}
