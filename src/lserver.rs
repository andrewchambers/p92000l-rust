use super::fcall;
use super::fcall::*;
use super::lerrno;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Read, Write};

pub trait Filesystem {
    type Fid;

    fn statfs(&self, _: &mut Self::Fid) -> Result<Rstatfs, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn lopen(&self, _: &mut Self::Fid, _flags: fcall::LOpenFlags) -> Result<Rlopen, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn lcreate(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _flags: u32,
        _mode: u32,
        _gid: u32,
    ) -> Result<Rlcreate, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn symlink(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _sym: &str,
        _gid: u32,
    ) -> Result<Rsymlink, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
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
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn rename(
        &self,
        _: &mut Self::Fid,
        _: &mut Self::Fid,
        _name: &str,
    ) -> Result<Rrename, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn readlink(&self, _: &mut Self::Fid) -> Result<Rreadlink<'static>, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn getattr(&self, _: &mut Self::Fid, _req_mask: GetattrMask) -> Result<Rgetattr, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn setattr(
        &self,
        _: &mut Self::Fid,
        _valid: SetattrMask,
        _stat: &SetAttr,
    ) -> Result<Rsetattr, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn xattrwalk(
        &self,
        _: &mut Self::Fid,
        _: &mut Self::Fid,
        _name: &str,
    ) -> Result<Rxattrwalk, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn xattrcreate(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _attr_size: u64,
        _flags: u32,
    ) -> Result<Rxattrcreate, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn readdir(
        &self,
        _: &mut Self::Fid,
        _offset: u64,
        _count: u32,
    ) -> Result<Rreaddir<'static>, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn fsync(&self, _: &mut Self::Fid) -> Result<Rfsync, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn lock(&self, _: &mut Self::Fid, _lock: &Flock) -> Result<Rlock, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn getlock(&self, _: &mut Self::Fid, _lock: &Getlock) -> Result<Rgetlock<'static>, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn link(&self, _: &mut Self::Fid, _: &mut Self::Fid, _name: &str) -> Result<Rlink, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn mkdir(
        &self,
        _: &mut Self::Fid,
        _name: &str,
        _mode: u32,
        _gid: u32,
    ) -> Result<Rmkdir, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn renameat(
        &self,
        _: &mut Self::Fid,
        _oldname: &str,
        _: &mut Self::Fid,
        _newname: &str,
    ) -> Result<Rrenameat, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn unlinkat(&self, _: &mut Self::Fid, _name: &str, _flags: u32) -> Result<Runlinkat, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn auth(
        &self,
        _uname: &str,
        _aname: &str,
        _n_uname: u32,
    ) -> Result<(Self::Fid, Rauth), Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn attach(
        &self,
        _afid: Option<&mut Self::Fid>,
        _uname: &str,
        _aname: &str,
        _n_uname: u32,
    ) -> Result<(Self::Fid, Rattach), Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn flush(&self) -> Result<Rflush, Rlerror> {
        Ok(Rflush {})
    }

    fn walk(
        &self,
        _: &mut Self::Fid,
        _wnames: &[Cow<'_, str>],
    ) -> Result<(Option<Self::Fid>, Rwalk), Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn read(&self, _: &mut Self::Fid, _offset: u64, _buf: &mut [u8]) -> Result<usize, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn write(&self, _: &mut Self::Fid, _offset: u64, _data: &[u8]) -> Result<Rwrite, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn clunk(&self, _: &mut Self::Fid) -> Result<Rclunk, Rlerror> {
        Ok(Rclunk {})
    }

    fn remove(&self, _: &mut Self::Fid) -> Result<Rremove, Rlerror> {
        Err(Rlerror {
            ecode: lerrno::EOPNOTSUPP,
        })
    }

    fn version(&self, msize: u32, ver: &str) -> Rversion<'static> {
        match ver {
            P92000L => Rversion {
                msize,
                version: Cow::from(ver.to_owned()),
            },
            _ => Rversion {
                msize,
                version: Cow::from("unknown"),
            },
        }
    }
}

pub fn serve_single_threaded<R, W, F>(r: &mut R, w: &mut W, fs: &mut F)
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
    match fcall::read(r, &mut mbuf) {
        Ok(fcall::TaggedFcall {
            tag: fcall::NOTAG,
            fcall:
                Fcall::Tversion(Tversion {
                    ref msize,
                    ref version,
                }),
        }) => {
            let rversion = fs.version(*msize, version);
            assert!(rversion.msize <= *msize);
            assert!(rversion.msize >= 512);
            mbuf.resize(rversion.msize as usize, 0);
            dbuf.resize((rversion.msize - fcall::IOHDRSZ) as usize, 0);
            if fcall::write(
                w,
                &mut mbuf,
                &fcall::TaggedFcall {
                    tag: fcall::NOTAG,
                    fcall: rversion.into(),
                },
            )
            .is_err()
            {
                return;
            }
        }
        Ok(_) => return,
        Err(_) => return,
    }

    while let Ok(req) = fcall::read(r, &mut mbuf) {
        macro_rules! get_fid {
            ($ident:ident, $e:expr) => {
                match fids.get_mut(&$ident) {
                    Some($ident) => $e,
                    None => Fcall::Rlerror(Rlerror {
                        ecode: lerrno::EBADF,
                    }),
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
                        Fcall::Rlerror(Rlerror {
                            ecode: lerrno::EBADF,
                        })
                    }
                } else {
                    Fcall::Rlerror(Rlerror {
                        ecode: lerrno::EBADF,
                    })
                }
            }};
        }

        let resp = match req.fcall {
            Fcall::Tstatfs(Tstatfs { fid }) => get_fid!(fid, fs.statfs(fid).into()),
            Fcall::Tlopen(Tlopen { fid, ref flags }) => {
                get_fid!(fid, fs.lopen(fid, *flags).into())
            }
            Fcall::Tlcreate(Tlcreate {
                fid,
                ref name,
                ref flags,
                ref mode,
                ref gid,
            }) => get_fid!(fid, fs.lcreate(fid, name, *flags, *mode, *gid).into()),
            Fcall::Tsymlink(Tsymlink {
                fid,
                ref name,
                ref symtgt,
                ref gid,
            }) => get_fid!(fid, fs.symlink(fid, name, symtgt, *gid).into()),
            Fcall::Tmknod(Tmknod {
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
            Fcall::Treadlink(Treadlink { fid }) => get_fid!(fid, fs.readlink(fid).into()),
            Fcall::Tgetattr(Tgetattr { fid, ref req_mask }) => {
                get_fid!(fid, fs.getattr(fid, *req_mask).into())
            }
            Fcall::Tsetattr(Tsetattr {
                fid,
                ref valid,
                ref stat,
            }) => get_fid!(fid, fs.setattr(fid, *valid, stat).into()),
            Fcall::Treaddir(Treaddir {
                fid,
                ref offset,
                ref count,
            }) => get_fid!(fid, fs.readdir(fid, *offset, *count).into()),
            Fcall::Tfsync(Tfsync { fid }) => get_fid!(fid, fs.fsync(fid).into()),
            Fcall::Tmkdir(Tmkdir {
                dfid,
                ref name,
                ref mode,
                ref gid,
            }) => get_fid!(dfid, fs.mkdir(dfid, name, *mode, *gid).into()),
            Fcall::Tflush(Tflush { .. }) => fs.flush().into(),
            Fcall::Tread(Tread {
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
                                fcall::Rread {
                                    data: Cow::from(swapbuf),
                                }
                                .into()
                            }
                            Err(rlerror) => rlerror.into(),
                        }
                    }
                    None => Fcall::Rlerror(fcall::Rlerror {
                        ecode: lerrno::EBADF,
                    }),
                }
            }
            Fcall::Twrite(Twrite {
                fid,
                ref offset,
                ref data,
            }) => get_fid!(fid, fs.write(fid, *offset, data).into()),
            Fcall::Tclunk(Tclunk { fid }) => {
                let r = get_fid!(fid, fs.clunk(fid).into());
                if let Fcall::Rclunk(_) = r {
                    fids.remove(&fid);
                }
                r
            }
            Fcall::Tremove(Tremove { fid }) => get_fid!(fid, fs.remove(fid).into()),
            Fcall::Trename(Trename {
                fid,
                dfid,
                ref name,
            }) => get_fids!(fid, dfid, fs.rename(fid, dfid, name).into()),
            Fcall::Tlink(Tlink {
                dfid,
                fid,
                ref name,
            }) => get_fids!(fid, dfid, fs.link(dfid, fid, name).into()),
            Fcall::Trenameat(Trenameat {
                olddfid,
                ref oldname,
                newdfid,
                ref newname,
            }) => get_fids!(
                olddfid,
                newdfid,
                fs.renameat(olddfid, oldname, newdfid, newname).into()
            ),
            Fcall::Tunlinkat(Tunlinkat {
                dfid,
                ref name,
                ref flags,
            }) => get_fid!(dfid, fs.unlinkat(dfid, name, *flags).into()),
            /*
            Fcall::Txattrwalk {
                fid,
                new_fid,
                ref name,
            } => new_fid!(new_fid, get_fid!(fid, fs.rxattrwalk(fid, new_fid, name))),
            Fcall::Txattrcreate {
                fid,
                ref name,
                ref attr_size,
                ref flags,
            } => get_fid!(fid, fs.rxattrcreate(fid, name, *attr_size, *flags)),
            */
            Fcall::Tlock(Tlock { fid, ref flock }) => get_fid!(fid, fs.lock(fid, flock).into()),
            Fcall::Tgetlock(Tgetlock { fid, ref flock }) => {
                get_fid!(fid, fs.getlock(fid, flock).into())
            }
            Fcall::Tauth(Tauth {
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
            Fcall::Tattach(Tattach {
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
            Fcall::Twalk(Twalk {
                fid,
                new_fid,
                ref wnames,
            }) => match fids.get_mut(&fid) {
                Some(fid) => match fs.walk(fid, wnames) {
                    Ok((None, rwalk)) => rwalk.into(),
                    Ok((Some(f), rwalk)) => {
                        fids.insert(new_fid, f);
                        rwalk.into()
                    }
                    Err(rlerror) => rlerror.into(),
                },
                None => Fcall::Rlerror(fcall::Rlerror {
                    ecode: lerrno::EBADF,
                }),
            },
            _ => Fcall::Rlerror(fcall::Rlerror {
                ecode: lerrno::EOPNOTSUPP,
            }),
        };

        let tagged_resp = fcall::TaggedFcall {
            tag: req.tag,
            fcall: resp,
        };

        if fcall::write(w, &mut mbuf, &tagged_resp).is_err() {
            break;
        }

        if let Fcall::Rread(Rread {
            data: Cow::Owned(mut data),
        }) = tagged_resp.fcall
        {
            // reclaim the data buffer.
            std::mem::swap(&mut dbuf, &mut data);
        }
    }
}
