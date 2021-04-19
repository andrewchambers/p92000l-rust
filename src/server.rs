use super::errno::*;
use super::fcall;
use super::fcall::*;
use std::collections::HashMap;
use std::io::{Read, Write};

pub trait Filesystem {
    type Fid;

    fn statfs(&self, _: &mut Self::Fid) -> Result<Rstatfs, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn lopen(&self, _: &mut Self::Fid, _flags: u32) -> Result<Rlopen, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn lcreate(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _flags: u32,
        _mode: u32,
        _gid: u32,
    ) -> Result<Rlcreate, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn symlink(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _sym: &str,
        _gid: u32,
    ) -> Result<Rsymlink, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn mknod(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _mode: u32,
        _major: u32,
        _minor: u32,
        _gid: u32,
    ) -> Result<Rmknod, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn rename(
        &self,
        _: &mut Self::Fid,
        _: &mut Self::Fid,
        _name: &str,
    ) -> Result<Rrename, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn readlink(&self, _: &mut Self::Fid) -> Result<Rreadlink, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn getattr(&self, _: &mut Self::Fid, _req_mask: GetattrMask) -> Result<Rgetattr, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn setattr(
        &self,
        _: &mut Self::Fid,
        _valid: SetattrMask,
        _stat: &SetAttr,
    ) -> Result<Rsetattr, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn xattrwalk(
        &self,
        _: &mut Self::Fid,
        _: &mut Self::Fid,
        _name: &str,
    ) -> Result<Rxattrwalk, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn xattrcreate(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _attr_size: u64,
        _flags: u32,
    ) -> Result<Rxattrcreate, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn readdir(&self, _: &mut Self::Fid, _offset: u64, _count: u32) -> Result<Rreaddir, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn fsync(&self, _: &mut Self::Fid) -> Result<Rfsync, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn lock(&self, _: &mut Self::Fid, _lock: &NcFlock) -> Result<Rlock, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn getlock(&self, _: &mut Self::Fid, _lock: &NcGetlock) -> Result<Rgetlock, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn link(&self, _: &mut Self::Fid, _: &mut Self::Fid, _name: &str) -> Result<Rlink, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn mkdir(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _mode: u32,
        _gid: u32,
    ) -> Result<Rmkdir, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn renameat(
        &self,
        _: &mut Self::Fid,
        _oldname: &str,
        _: &mut Self::Fid,
        _newname: &str,
    ) -> Result<Rrenameat, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn unlinkat(&self, _: &mut Self::Fid, _name: &str, _flags: u32) -> Result<Runlinkat, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn auth(
        &self,
        _uname: &str,
        _aname: &str,
        _n_uname: u32,
    ) -> Result<(Self::Fid, Rauth), Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn attach(
        &self,
        _afid: Option<&mut Self::Fid>,
        _uname: &str,
        _aname: &str,
        _n_uname: u32,
    ) -> Result<(Self::Fid, Rattach), Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn flush(&self) -> Result<Rflush, Rlerror> {
        Ok(Rflush {})
    }

    fn walk(
        &self,
        _: &mut Self::Fid,
        _wnames: &[&str],
    ) -> Result<(Option<Self::Fid>, Rwalk), Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn read(&self, _: &mut Self::Fid, _offset: u64, _buf: &mut [u8]) -> Result<usize, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn write(&self, _: &mut Self::Fid, _offset: u64, _data: &[u8]) -> Result<Rwrite, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn clunk(&self, _: &mut Self::Fid) -> Result<Rclunk, Rlerror> {
        Ok(Rclunk {})
    }

    fn remove(&self, _: &mut Self::Fid) -> Result<Rremove, Rlerror> {
        Err(Rlerror { ecode: EOPNOTSUPP })
    }

    fn version(&self, msize: u32, ver: &str) -> Rversion {
        match ver {
            P92000L => Rversion {
                msize,
                version: ver.to_owned(),
            },
            _ => Rversion {
                msize,
                version: "unknown".to_owned(),
            },
        }
    }
}

pub fn serve_single_threaded<R, W, F>(r: &mut R, w: &mut W, fs: &mut F) -> ()
where
    R: Read,
    W: Write,
    F: Filesystem,
{
    let mut fids = HashMap::<u32, F::Fid>::new();
    // Message buffer.
    let mut mbuf: Vec<u8> = Vec::with_capacity(65536);
    // Data buffer.
    let mut dbuf: Vec<u8> = Vec::with_capacity(8192);

    // Handle version and size buffers.
    match fcall::read_msg(r, &mut mbuf) {
        Ok(fcall::NcMsg {
            tag: fcall::NOTAG,
            body:
                NcFcall::Tversion(NcTversion {
                    ref msize,
                    ref version,
                }),
        }) => {
            let rversion = fs.version(*msize, version);
            assert!(rversion.msize <= *msize);
            mbuf.resize(rversion.msize as usize, 0);
            if !write_msg(
                w,
                &mut mbuf,
                &fcall::Msg {
                    tag: fcall::NOTAG,
                    body: rversion.into(),
                },
            )
            .is_ok()
            {
                return ();
            }
        }
        Ok(_) => return (),
        Err(_) => return (),
    }

    while let Ok(msg) = fcall::read_msg(r, &mut mbuf) {
        // dbg!(&msg);

        macro_rules! get_fid {
            ($ident:ident, $e:expr) => {
                match fids.get_mut(&$ident) {
                    Some($ident) => $e,
                    None => Fcall::Rlerror(Rlerror { ecode: EBADF }),
                }
            };
        }

        macro_rules! get_fids {
            ($f1:ident, $f2:ident, $e:expr) => {{
                // Work around borrow restrictions by removing the fid temporarily,
                // then re-adding it after we are done.
                let f1 = $f1;
                if let Some(mut $f1) = fids.remove(&f1) {
                    if let Some($f2) = fids.get_mut(&$f2) {
                        let $f1 = &mut $f1;
                        $e
                    } else {
                        fids.insert(f1, $f1);
                        Fcall::Rlerror(Rlerror { ecode: EBADF })
                    }
                } else {
                    Fcall::Rlerror(Rlerror { ecode: EBADF })
                }
            }};
        }

        let resp = match msg.body {
            NcFcall::Tstatfs(Tstatfs { fid }) => get_fid!(fid, fs.statfs(fid).into()),
            NcFcall::Tlopen(Tlopen { fid, ref flags }) => {
                get_fid!(fid, fs.lopen(fid, *flags).into())
            }
            NcFcall::Tlcreate(NcTlcreate {
                fid,
                ref name,
                ref flags,
                ref mode,
                ref gid,
            }) => get_fid!(fid, fs.lcreate(fid, name, *flags, *mode, *gid).into()),
            NcFcall::Tsymlink(NcTsymlink {
                fid,
                ref name,
                ref symtgt,
                ref gid,
            }) => get_fid!(fid, fs.symlink(fid, name, symtgt, *gid).into()),
            NcFcall::Tmknod(NcTmknod {
                dfid,
                ref name,
                ref mode,
                ref major,
                ref minor,
                ref gid,
            }) => get_fid!(
                dfid,
                fs.mknod(dfid, name, *mode, *major, *minor, *gid).into()
            ),
            NcFcall::Treadlink(Treadlink { fid }) => get_fid!(fid, fs.readlink(fid).into()),
            NcFcall::Tgetattr(Tgetattr { fid, ref req_mask }) => {
                get_fid!(fid, fs.getattr(fid, *req_mask).into())
            }
            NcFcall::Tsetattr(Tsetattr {
                fid,
                ref valid,
                ref stat,
            }) => get_fid!(fid, fs.setattr(fid, *valid, stat).into()),
            NcFcall::Treaddir(Treaddir {
                fid,
                ref offset,
                ref count,
            }) => get_fid!(fid, fs.readdir(fid, *offset, *count).into()),
            NcFcall::Tfsync(Tfsync { fid }) => get_fid!(fid, fs.fsync(fid).into()),
            NcFcall::Tmkdir(NcTmkdir {
                dfid,
                ref name,
                ref mode,
                ref gid,
            }) => get_fid!(dfid, fs.mkdir(dfid, name, *mode, *gid).into()),
            NcFcall::Tflush(Tflush { .. }) => fs.flush().into(),
            NcFcall::Tread(Tread {
                fid,
                ref offset,
                ref count,
            }) => {
                match fids.get_mut(&fid) {
                    Some(fid) => {
                        let count = *count as usize;
                        let count = count.min(dbuf.capacity());
                        // This is safe as we just checked the count against the capacity.
                        unsafe { dbuf.set_len(count) };
                        match fs.read(fid, *offset, &mut dbuf[..]) {
                            Ok(n) => {
                                // Temporarily borrow the data buffer, we swap it back later.
                                dbuf.truncate(n);
                                let mut swapbuf = Vec::new();
                                std::mem::swap(&mut dbuf, &mut swapbuf);
                                fcall::Rread { data: swapbuf }.into()
                            }
                            Err(rlerror) => rlerror.into(),
                        }
                    }
                    None => Fcall::Rlerror(fcall::Rlerror { ecode: EBADF }),
                }
            }
            NcFcall::Twrite(NcTwrite {
                fid,
                ref offset,
                ref data,
            }) => get_fid!(fid, fs.write(fid, *offset, data).into()),
            NcFcall::Tclunk(Tclunk { fid }) => {
                let r = get_fid!(fid, fs.clunk(fid).into());
                if let Fcall::Rclunk(_) = r {
                    fids.remove(&fid);
                }
                r
            }
            NcFcall::Tremove(Tremove { fid }) => get_fid!(fid, fs.remove(fid).into()),
            NcFcall::Trename(NcTrename {
                fid,
                dfid,
                ref name,
            }) => get_fids!(fid, dfid, fs.rename(fid, dfid, name).into()),
            NcFcall::Tlink(NcTlink {
                dfid,
                fid,
                ref name,
            }) => get_fids!(fid, dfid, fs.link(dfid, fid, name).into()),
            NcFcall::Trenameat(NcTrenameat {
                olddirfid,
                ref oldname,
                newdirfid,
                ref newname,
            }) => get_fids!(
                olddirfid,
                newdirfid,
                fs.renameat(olddirfid, oldname, newdirfid, newname).into()
            ),
            NcFcall::Tunlinkat(NcTunlinkat {
                dirfd,
                ref name,
                ref flags,
            }) => get_fid!(dirfd, fs.unlinkat(dirfd, name, *flags).into()),
            /*
            NcFcall::Txattrwalk {
                fid,
                newfid,
                ref name,
            } => new_fid!(newfid, get_fid!(fid, fs.rxattrwalk(fid, newfid, name))),
            NcFcall::Txattrcreate {
                fid,
                ref name,
                ref attr_size,
                ref flags,
            } => get_fid!(fid, fs.rxattrcreate(fid, name, *attr_size, *flags)),
            */
            NcFcall::Tlock(NcTlock { fid, ref flock }) => get_fid!(fid, fs.lock(fid, flock).into()),
            NcFcall::Tgetlock(NcTgetlock { fid, ref flock }) => {
                get_fid!(fid, fs.getlock(fid, flock).into())
            }
            NcFcall::Tauth(NcTauth {
                afid,
                ref uname,
                ref aname,
                ref n_uname,
            }) => match fs.auth(uname, aname, *n_uname) {
                Ok((f, rauth)) => {
                    fids.insert(afid, f);
                    rauth.into()
                }
                Err(rlerror) => rlerror.into(),
            },
            NcFcall::Tattach(NcTattach {
                fid,
                afid: _,
                ref uname,
                ref aname,
                ref n_uname,
            }) => match fs.attach(None, uname, aname, *n_uname) {
                Ok((f, rattach)) => {
                    fids.insert(fid, f);
                    rattach.into()
                }
                Err(rlerror) => rlerror.into(),
            },
            NcFcall::Twalk(NcTwalk {
                fid,
                newfid,
                ref wnames,
            }) => match fids.get_mut(&fid) {
                Some(fid) => match fs.walk(fid, wnames) {
                    Ok((None, rwalk)) => rwalk.into(),
                    Ok((Some(f), rwalk)) => {
                        fids.insert(newfid, f);
                        rwalk.into()
                    }
                    Err(rlerror) => rlerror.into(),
                },
                None => Fcall::Rlerror(fcall::Rlerror { ecode: EBADF }),
            },
            _ => Fcall::Rlerror(fcall::Rlerror { ecode: EOPNOTSUPP }),
        };

        let resp_msg = fcall::Msg {
            tag: msg.tag,
            body: resp,
        };

        // dbg!(&resp_msg);
        if !fcall::write_msg(w, &mut mbuf, &resp_msg).is_ok() {
            break;
        }

        match resp_msg.body {
            Fcall::Rread(Rread { mut data }) => {
                // reclaim the data buffer.
                std::mem::swap(&mut dbuf, &mut data);
            }
            _ => (),
        }
    }
}
