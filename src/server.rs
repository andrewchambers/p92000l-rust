use super::errno::*;
use super::fcall;
use super::fcall::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::io::{Read, Write};

pub trait Filesystem {
    type Fid: Default;

    fn rstatfs(&self, _: &Self::Fid) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rlopen(&self, _: &Self::Fid, _flags: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rlcreate(&self, _: &Self::Fid, _name: &str, _flags: u32, _mode: u32, _gid: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rsymlink(&self, _: &Self::Fid, _name: &str, _sym: &str, _gid: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rmknod(
        &self,
        _: &Self::Fid,
        _name: &str,
        _mode: u32,
        _major: u32,
        _minor: u32,
        _gid: u32,
    ) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rrename(&self, _: &Self::Fid, _: &Self::Fid, _name: &str) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rreadlink(&self, _: &Self::Fid) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rgetattr(&self, _: &Self::Fid, _req_mask: GetattrMask) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rsetattr(&self, _: &Self::Fid, _valid: SetattrMask, _stat: &SetAttr) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rxattrwalk(&self, _: &Self::Fid, _: &Self::Fid, _name: &str) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rxattrcreate(&self, _: &Self::Fid, _name: &str, _attr_size: u64, _flags: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rreaddir(&self, _: &Self::Fid, _offset: u64, _count: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rfsync(&self, _: &Self::Fid) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rlock(&self, _: &Self::Fid, _lock: &Flock) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rgetlock(&self, _: &Self::Fid, _lock: &Getlock) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rlink(&self, _: &Self::Fid, _: &Self::Fid, _name: &str) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rmkdir(&self, _: &Self::Fid, _name: &str, _mode: u32, _gid: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rrenameat(&self, _: &Self::Fid, _oldname: &str, _: &Self::Fid, _newname: &str) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn runlinkat(&self, _: &Self::Fid, _name: &str, _flags: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rauth(&self, _: &Self::Fid, _uname: &str, _aname: &str, _n_uname: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rattach(
        &self,
        _: &Self::Fid,
        _afid: Option<&Self::Fid>,
        _uname: &str,
        _aname: &str,
        _n_uname: u32,
    ) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rflush(&self, _old: Option<&Fcall>) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rwalk(&self, _: &Self::Fid, _new: &Self::Fid, _wnames: &[String]) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rread(&self, _: &Self::Fid, _offset: u64, _count: u32) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rwrite(&self, _: &Self::Fid, _offset: u64, _data: &Data) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rclunk(&self, _: &Self::Fid) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rremove(&self, _: &Self::Fid) -> Fcall {
        Fcall::Rlerror { ecode: EOPNOTSUPP }
    }

    fn rversion(&self, msize: u32, ver: &str) -> Fcall {
        match ver {
            P92000L => Fcall::Rversion {
                msize,
                version: ver.to_owned(),
            },
            _ => Fcall::Rlerror {
                ecode: EPROTONOSUPPORT,
            },
        }
    }
}

pub fn serve_single_threaded<R, W, F>(r: &mut R, w: &mut W, fs: &mut F) -> ()
where
    R: Read + ReadBytesExt,
    W: Write + WriteBytesExt,
    F: Filesystem,
{
    let mut fids = HashMap::<u32, F::Fid>::new();

    let mut buf: Vec<u8> = Vec::with_capacity(65536);

    while let Ok(sz) = r.read_u32::<LittleEndian>() {
        let sz = (sz as usize) - 4;
        buf.resize(sz, 0);
        if !r.read_exact(&mut buf[..]).is_ok() {
            break;
        }
        let mut msg = match fcall::read_msg(&mut std::io::Cursor::new(&buf)) {
            Ok(msg) => msg,
            _ => break,
        };

        macro_rules! new_fid {
            ($f:ident, $e:expr) => {{
                let f = $f;
                let mut fv = Default::default();
                let $f = &mut fv;
                let r = $e;
                if !matches!(r, Fcall::Rlerror { .. }) {
                    fids.insert(f, fv);
                }
                r
            }};
        }

        macro_rules! get_fid {
            ($f:ident, $e:expr) => {
                if let Some($f) = fids.get_mut(&$f) {
                    $e
                } else {
                    Fcall::Rlerror { ecode: EOPNOTSUPP }
                };
            };
        }

        macro_rules! get_fids {
            ($f1:ident, $f2:ident, $e:expr) => {{
                // Work around borrow restrictions by removing the fid temporarily,
                // then re-adding it.
                let f1 = $f1;
                if let Some(mut $f1) = fids.remove(&f1) {
                    if let Some($f2) = fids.get_mut(&$f2) {
                        let $f1 = &mut $f1;
                        $e
                    } else {
                        fids.insert(f1, $f1);
                        Fcall::Rlerror { ecode: EOPNOTSUPP }
                    }
                } else {
                    Fcall::Rlerror { ecode: EOPNOTSUPP }
                }
            }};
        }

        msg.body = match msg.body {
            Fcall::Tstatfs { fid } => get_fid!(fid, fs.rstatfs(fid)),
            Fcall::Tlopen { fid, ref flags } => get_fid!(fid, fs.rlopen(fid, *flags)),
            Fcall::Tlcreate {
                fid,
                ref name,
                ref flags,
                ref mode,
                ref gid,
            } => get_fid!(fid, fs.rlcreate(fid, name, *flags, *mode, *gid)),
            Fcall::Tsymlink {
                fid,
                ref name,
                ref symtgt,
                ref gid,
            } => get_fid!(fid, fs.rsymlink(fid, name, symtgt, *gid)),
            Fcall::Tmknod {
                dfid,
                ref name,
                ref mode,
                ref major,
                ref minor,
                ref gid,
            } => get_fid!(dfid, fs.rmknod(dfid, name, *mode, *major, *minor, *gid)),
            Fcall::Treadlink { fid } => get_fid!(fid, fs.rreadlink(fid)),
            Fcall::Tgetattr { fid, ref req_mask } => get_fid!(fid, fs.rgetattr(fid, *req_mask)),
            Fcall::Tsetattr {
                fid,
                ref valid,
                ref stat,
            } => get_fid!(fid, fs.rsetattr(fid, *valid, stat)),
            Fcall::Treaddir {
                fid,
                ref offset,
                ref count,
            } => get_fid!(fid, fs.rreaddir(fid, *offset, *count)),
            Fcall::Tfsync { fid } => get_fid!(fid, fs.rfsync(fid)),
            Fcall::Tmkdir {
                dfid,
                ref name,
                ref mode,
                ref gid,
            } => get_fid!(dfid, fs.rmkdir(dfid, name, *mode, *gid)),
            Fcall::Tversion {
                ref msize,
                ref version,
            } => fs.rversion(*msize, version),
            Fcall::Tflush { oldtag: _ } => fs.rflush(None),
            Fcall::Tread {
                fid,
                ref offset,
                ref count,
            } => get_fid!(fid, fs.rread(fid, *offset, *count)),
            Fcall::Twrite {
                fid,
                ref offset,
                ref data,
            } => get_fid!(fid, fs.rwrite(fid, *offset, data)),
            Fcall::Tclunk { fid } => get_fid!(fid, fs.rclunk(fid)),
            Fcall::Tremove { fid } => get_fid!(fid, fs.rremove(fid)),
            Fcall::Trename {
                fid,
                dfid,
                ref name,
            } => get_fids!(fid, dfid, fs.rrename(fid, dfid, name)),
            Fcall::Tlink {
                dfid,
                fid,
                ref name,
            } => get_fids!(fid, dfid, fs.rlink(dfid, fid, name)),
            Fcall::Trenameat {
                olddirfid,
                ref oldname,
                newdirfid,
                ref newname,
            } => get_fids!(
                olddirfid,
                newdirfid,
                fs.rrenameat(olddirfid, oldname, newdirfid, newname)
            ),
            Fcall::Tunlinkat {
                dirfd,
                ref name,
                ref flags,
            } => get_fid!(dirfd, fs.runlinkat(dirfd, name, *flags)),
            Fcall::Txattrwalk {
                fid,
                newfid,
                ref name,
            } => new_fid!(newfid, get_fid!(fid, fs.rxattrwalk(fid, newfid, name))),
            Fcall::Txattrcreate {
                fid,
                ref name,
                ref attr_size,
                ref flags,
            } => get_fid!(fid, fs.rxattrcreate(fid, name, *attr_size, *flags)),
            Fcall::Tlock { fid, ref flock } => get_fid!(fid, fs.rlock(fid, flock)),
            Fcall::Tgetlock { fid, ref flock } => get_fid!(fid, fs.rgetlock(fid, flock)),
            Fcall::Tauth {
                afid,
                ref uname,
                ref aname,
                ref n_uname,
            } => new_fid!(afid, fs.rauth(afid, uname, aname, *n_uname)),
            Fcall::Tattach {
                fid,
                afid: _,
                ref uname,
                ref aname,
                ref n_uname,
            } => new_fid!(fid, fs.rattach(fid, None, uname, aname, *n_uname)),
            Fcall::Twalk {
                fid,
                newfid,
                ref wnames,
            } => new_fid!(newfid, get_fid!(fid, fs.rwalk(fid, newfid, wnames))),
            _ => Fcall::Rlerror { ecode: EOPNOTSUPP },
        };

        if let Fcall::Tclunk { fid } = msg.body {
            fids.remove(&fid);
        }

        buf.resize(0, 0);
        match fcall::write_msg(&mut std::io::Cursor::new(&mut buf), &mut msg) {
            Ok(msg) => msg,
            _ => break,
        };

        if !w.write_all(&buf[..]).is_ok() {
            break;
        };
    }
}
