use bitflags::bitflags;
use std::borrow::Cow;
use std::convert::TryInto;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;

/// Special tag which `Tversion`/`Rversion` must use as `tag`
pub const NOTAG: u16 = !0;

/// Special value which `Tattach` with no auth must use as `afid`
///
/// If the client does not wish to authenticate the connection, or knows that authentication is
/// not required, the afid field in the attach message should be set to `NOFID`
pub const NOFID: u32 = !0;

/// Special uid which `Tauth`/`Tattach` use as `n_uname` to indicate no uid is specified
pub const NONUNAME: u32 = !0;

/// Room for `Twrite`/`Rread` header
///
/// size[4] Tread/Twrite[2] tag[2] fid[4] offset[8] count[4]
pub const IOHDRSZ: u32 = 24;

/// Room for readdir header
pub const READDIRHDRSZ: u32 = 24;

/// Maximum elements in a single walk.
pub const MAXWELEM: usize = 13;

bitflags! {
    /// Flags passed to Tlopen.
    pub struct LOpenFlags: u32 {
        const O_RDONLY    = 0;
        const O_WRONLY    = 1;
        const O_RDWR    = 2;
        const O_EXCL = 0o200;
        const O_TRUNC = 0o1000;
    }
}

bitflags! {
    /// File lock type, Flock.typ
    pub struct LockType: u8 {
        const RDLOCK    = 0;
        const WRLOCK    = 1;
        const UNLOCK    = 2;
    }
}

bitflags! {
    /// File lock flags, Flock.flags
    pub struct LockFlag: u32 {
        #[doc = "Blocking request"]
        const BLOCK     = 1;
        #[doc = "Reserved for future use"]
        const RECLAIM   = 2;
    }
}

bitflags! {
    /// File lock status
    pub struct LockStatus: u8 {
        const SUCCESS   = 0;
        const BLOCKED   = 1;
        const ERROR     = 2;
        const GRACE     = 3;
    }
}

bitflags! {
    /// Bits in Qid.typ
    ///
    /// QidType can be constructed from std::fs::FileType via From trait
    ///
    /// # Protocol
    /// 9P2000/9P2000.L
    #[derive(Default)]
    pub struct QidType: u8 {
        #[doc = "Type bit for directories"]
        const DIR       = 0x80;
        #[doc = "Type bit for append only files"]
        const APPEND    = 0x40;
        #[doc = "Type bit for exclusive use files"]
        const EXCL      = 0x20;
        #[doc = "Type bit for mounted channel"]
        const MOUNT     = 0x10;
        #[doc = "Type bit for authentication file"]
        const AUTH      = 0x08;
        #[doc = "Type bit for not-backed-up file"]
        const TMP       = 0x04;
        #[doc = "Type bits for symbolic links (9P2000.u)"]
        const SYMLINK   = 0x02;
        #[doc = "Type bits for hard-link (9P2000.u)"]
        const LINK      = 0x01;
        #[doc = "Plain file"]
        const FILE      = 0x00;
    }
}

impl From<::std::fs::FileType> for QidType {
    fn from(typ: ::std::fs::FileType) -> Self {
        From::from(&typ)
    }
}

impl<'a> From<&'a ::std::fs::FileType> for QidType {
    fn from(typ: &'a ::std::fs::FileType) -> Self {
        let mut qid_type = QidType::FILE;

        if typ.is_dir() {
            qid_type.insert(QidType::DIR)
        }

        if typ.is_symlink() {
            qid_type.insert(QidType::SYMLINK)
        }

        qid_type
    }
}

bitflags! {
    /// Bits in `mask` and `valid` of `Tgetattr` and `Rgetattr`.
    ///
    /// # Protocol
    /// 9P2000.L
    pub struct GetattrMask: u64 {
        const MODE          = 0x00000001;
        const NLINK         = 0x00000002;
        const UID           = 0x00000004;
        const GID           = 0x00000008;
        const RDEV          = 0x00000010;
        const ATIME         = 0x00000020;
        const MTIME         = 0x00000040;
        const CTIME         = 0x00000080;
        const INO           = 0x00000100;
        const SIZE          = 0x00000200;
        const BLOCKS        = 0x00000400;

        const BTIME         = 0x00000800;
        const GEN           = 0x00001000;
        const DATA_VERSION  = 0x00002000;

        #[doc = "Mask for fields up to BLOCKS"]
        const BASIC         =0x000007ff;
        #[doc = "Mask for All fields above"]
        const ALL           = 0x00003fff;
    }
}

bitflags! {
    /// Bits in `mask` of `Tsetattr`.
    ///
    /// If a time bit is set without the corresponding SET bit, the current
    /// system time on the server is used instead of the value sent in the request.
    ///
    /// # Protocol
    /// 9P2000.L
    pub struct SetattrMask: u32 {
        const MODE      = 0x00000001;
        const UID       = 0x00000002;
        const GID       = 0x00000004;
        const SIZE      = 0x00000008;
        const ATIME     = 0x00000010;
        const MTIME     = 0x00000020;
        const CTIME     = 0x00000040;
        const ATIME_SET = 0x00000080;
        const MTIME_SET = 0x00000100;
    }
}

impl From<nix::sys::statvfs::Statvfs> for Statfs {
    fn from(buf: nix::sys::statvfs::Statvfs) -> Statfs {
        Statfs {
            typ: 0,
            bsize: buf.block_size() as u32,
            blocks: buf.blocks(),
            bfree: buf.blocks_free(),
            bavail: buf.blocks_available(),
            files: buf.files(),
            ffree: buf.files_free(),
            fsid: buf.filesystem_id() as u64,
            namelen: buf.name_max() as u32,
        }
    }
}

impl From<fs::Metadata> for Stat {
    fn from(attr: fs::Metadata) -> Self {
        From::from(&attr)
    }
}

impl<'a> From<&'a fs::Metadata> for Stat {
    fn from(attr: &'a fs::Metadata) -> Self {
        Stat {
            mode: attr.mode(),
            uid: attr.uid(),
            gid: attr.gid(),
            nlink: attr.nlink(),
            rdev: attr.rdev(),
            size: attr.size() as u64,
            blksize: attr.blksize() as u64,
            blocks: attr.blocks() as u64,
            atime: Time {
                sec: attr.atime() as u64,
                nsec: attr.atime_nsec() as u64,
            },
            mtime: Time {
                sec: attr.mtime() as u64,
                nsec: attr.mtime_nsec() as u64,
            },
            ctime: Time {
                sec: attr.ctime() as u64,
                nsec: attr.ctime_nsec() as u64,
            },
            btime: Time { sec: 0, nsec: 0 },
            gen: 0,
            data_version: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirEntryData<'a> {
    pub data: Vec<DirEntry<'a>>,
}

impl<'a> DirEntryData<'a> {
    pub fn new() -> DirEntryData<'a> {
        Self::with(Vec::new())
    }
    pub fn with(v: Vec<DirEntry<'a>>) -> DirEntryData<'a> {
        DirEntryData { data: v }
    }
    pub fn data(&self) -> &[DirEntry] {
        &self.data
    }
    pub fn size(&self) -> u64 {
        self.data.iter().fold(0, |a, e| a + e.size()) as u64
    }
    pub fn push(&mut self, entry: DirEntry<'a>) {
        self.data.push(entry);
    }
}

impl<'b> Default for DirEntryData<'b> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FcallType {
    // 9P2000.L
    Rlerror = 7,
    Tstatfs = 8,
    Rstatfs,
    Tlopen = 12,
    Rlopen,
    Tlcreate = 14,
    Rlcreate,
    Tsymlink = 16,
    Rsymlink,
    Tmknod = 18,
    Rmknod,
    Trename = 20,
    Rrename,
    Treadlink = 22,
    Rreadlink,
    Tgetattr = 24,
    Rgetattr,
    Tsetattr = 26,
    Rsetattr,
    Txattrwalk = 30,
    Rxattrwalk,
    Txattrcreate = 32,
    Rxattrcreate,
    Treaddir = 40,
    Rreaddir,
    Tfsync = 50,
    Rfsync,
    Tlock = 52,
    Rlock,
    Tgetlock = 54,
    Rgetlock,
    Tlink = 70,
    Rlink,
    Tmkdir = 72,
    Rmkdir,
    Trenameat = 74,
    Rrenameat,
    Tunlinkat = 76,
    Runlinkat,

    // 9P2000
    Tversion = 100,
    Rversion,
    Tauth = 102,
    Rauth,
    Tattach = 104,
    Rattach,
    //Terror          = 106,  // Illegal, never used
    //Rerror,
    Tflush = 108,
    Rflush,
    Twalk = 110,
    Rwalk,
    //Topen           = 112,
    //Ropen,
    //Tcreate         = 114,
    //Rcreate,
    Tread = 116,
    Rread,
    Twrite = 118,
    Rwrite,
    Tclunk = 120,
    Rclunk,
    Tremove = 122,
    Rremove,
    //Tstat           = 124,
    //Rstat,
    //Twstat          = 126,
    //Rwstat,
}

impl FcallType {
    pub fn from_u8(v: u8) -> Option<FcallType> {
        match v {
            // 9P2000.L
            // 6 => Some(FcallType::Tlerror),
            7 => Some(FcallType::Rlerror),
            8 => Some(FcallType::Tstatfs),
            9 => Some(FcallType::Rstatfs),
            12 => Some(FcallType::Tlopen),
            13 => Some(FcallType::Rlopen),
            14 => Some(FcallType::Tlcreate),
            15 => Some(FcallType::Rlcreate),
            16 => Some(FcallType::Tsymlink),
            17 => Some(FcallType::Rsymlink),
            18 => Some(FcallType::Tmknod),
            19 => Some(FcallType::Rmknod),
            20 => Some(FcallType::Trename),
            21 => Some(FcallType::Rrename),
            22 => Some(FcallType::Treadlink),
            23 => Some(FcallType::Rreadlink),
            24 => Some(FcallType::Tgetattr),
            25 => Some(FcallType::Rgetattr),
            26 => Some(FcallType::Tsetattr),
            27 => Some(FcallType::Rsetattr),
            30 => Some(FcallType::Txattrwalk),
            31 => Some(FcallType::Rxattrwalk),
            32 => Some(FcallType::Txattrcreate),
            33 => Some(FcallType::Rxattrcreate),
            40 => Some(FcallType::Treaddir),
            41 => Some(FcallType::Rreaddir),
            50 => Some(FcallType::Tfsync),
            51 => Some(FcallType::Rfsync),
            52 => Some(FcallType::Tlock),
            53 => Some(FcallType::Rlock),
            54 => Some(FcallType::Tgetlock),
            55 => Some(FcallType::Rgetlock),
            70 => Some(FcallType::Tlink),
            71 => Some(FcallType::Rlink),
            72 => Some(FcallType::Tmkdir),
            73 => Some(FcallType::Rmkdir),
            74 => Some(FcallType::Trenameat),
            75 => Some(FcallType::Rrenameat),
            76 => Some(FcallType::Tunlinkat),
            77 => Some(FcallType::Runlinkat),

            // 9P2000
            100 => Some(FcallType::Tversion),
            101 => Some(FcallType::Rversion),
            102 => Some(FcallType::Tauth),
            103 => Some(FcallType::Rauth),
            104 => Some(FcallType::Tattach),
            105 => Some(FcallType::Rattach),
            // 106 => Some(FcallType::Terror),
            // 107 => Some(FcallType::Rerror),
            108 => Some(FcallType::Tflush),
            109 => Some(FcallType::Rflush),
            110 => Some(FcallType::Twalk),
            111 => Some(FcallType::Rwalk),
            // 112 => Some(FcallType::Topen),
            // 113 => Some(FcallType::Ropen),
            // 114 => Some(FcallType::Tcreate),
            // 115 => Some(FcallType::Rcreate ),
            116 => Some(FcallType::Tread),
            117 => Some(FcallType::Rread),
            118 => Some(FcallType::Twrite),
            119 => Some(FcallType::Rwrite),
            120 => Some(FcallType::Tclunk),
            121 => Some(FcallType::Rclunk),
            122 => Some(FcallType::Tremove),
            123 => Some(FcallType::Rremove),
            // 124 => Some(FcallType::Tstat),
            // 125 => Some(FcallType::Rstat),
            // 126 => Some(FcallType::Twstat),
            // 126 => Some(FcallType::Rwstat),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Qid {
    pub typ: QidType,
    pub version: u32,
    pub path: u64,
}
#[derive(Clone, Debug, Copy)]
pub struct Statfs {
    pub typ: u32,
    pub bsize: u32,
    pub blocks: u64,
    pub bfree: u64,
    pub bavail: u64,
    pub files: u64,
    pub ffree: u64,
    pub fsid: u64,
    pub namelen: u32,
}
#[derive(Clone, Debug, Copy)]
pub struct Time {
    pub sec: u64,
    pub nsec: u64,
}
#[derive(Clone, Debug, Copy)]
pub struct Stat {
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub nlink: u64,
    pub rdev: u64,
    pub size: u64,
    pub blksize: u64,
    pub blocks: u64,
    pub atime: Time,
    pub mtime: Time,
    pub ctime: Time,
    pub btime: Time,
    pub gen: u64,
    pub data_version: u64,
}
#[derive(Clone, Debug, Copy)]
pub struct SetAttr {
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime: Time,
    pub mtime: Time,
}
#[derive(Clone, Debug)]
pub struct DirEntry<'a> {
    pub qid: Qid,
    pub offset: u64,
    pub typ: u8,
    pub name: Cow<'a, str>,
}

impl<'a> DirEntry<'a> {
    pub fn size(&self) -> u64 {
        (13 + 8 + 1 + 2 + self.name.len()) as u64
    }
}

#[derive(Clone, Debug)]
pub struct Flock<'a> {
    pub typ: LockType,
    pub flags: LockFlag,
    pub start: u64,
    pub length: u64,
    pub proc_id: u32,
    pub client_id: Cow<'a, str>,
}

#[derive(Clone, Debug)]
pub struct Getlock<'a> {
    pub typ: LockType,
    pub start: u64,
    pub length: u64,
    pub proc_id: u32,
    pub client_id: Cow<'a, str>,
}

#[derive(Clone, Debug)]
pub struct Rlerror {
    pub ecode: u32,
}

impl Rlerror {
    pub fn into_io_error(self) -> std::io::Error {
        use super::errno;
        use std::io::Error;
        use std::io::ErrorKind::*;

        match self.ecode {
            errno::ENOENT => Error::from(NotFound),
            errno::EPERM => Error::from(PermissionDenied),
            errno::ECONNREFUSED => Error::from(ConnectionRefused),
            errno::ECONNRESET => Error::from(ConnectionReset),
            errno::ECONNABORTED => Error::from(ConnectionAborted),
            errno::ENOTCONN => Error::from(NotConnected),
            errno::EADDRINUSE => Error::from(AddrInUse),
            errno::EADDRNOTAVAIL => Error::from(AddrNotAvailable),
            errno::EPIPE => Error::from(BrokenPipe),
            errno::EALREADY => Error::from(AlreadyExists),
            errno::EINVAL => Error::from(InvalidInput),
            errno::ETIMEDOUT => Error::from(TimedOut),
            errno::EINTR => Error::from(Interrupted),
            ecode => Error::new(Other, errno::strerror(ecode)),
        }
    }
}

impl From<std::io::Error> for Rlerror {
    fn from(err: std::io::Error) -> Self {
        use super::errno;
        use std::io::ErrorKind::*;

        let ecode = match err.kind() {
            NotFound => errno::ENOENT,
            PermissionDenied => errno::EPERM,
            ConnectionRefused => errno::ECONNREFUSED,
            ConnectionReset => errno::ECONNRESET,
            ConnectionAborted => errno::ECONNABORTED,
            NotConnected => errno::ENOTCONN,
            AddrInUse => errno::EADDRINUSE,
            AddrNotAvailable => errno::EADDRNOTAVAIL,
            BrokenPipe => errno::EPIPE,
            AlreadyExists => errno::EALREADY,
            WouldBlock => errno::EAGAIN,
            InvalidInput => errno::EINVAL,
            InvalidData => errno::EINVAL,
            TimedOut => errno::ETIMEDOUT,
            WriteZero => errno::EAGAIN,
            Interrupted => errno::EINTR,
            _ => errno::EIO,
        };

        Rlerror { ecode }
    }
}

#[derive(Clone, Debug)]
pub struct Tattach<'a> {
    pub fid: u32,
    pub afid: u32,
    pub uname: Cow<'a, str>,
    pub aname: Cow<'a, str>,
    pub n_uname: u32,
}

impl<'a> Tattach<'a> {
    pub fn clone_static(&self) -> Tattach<'static> {
        Tattach {
            afid: self.afid,
            fid: self.fid,
            n_uname: self.n_uname.to_owned(),
            aname: Cow::from(self.aname.clone().into_owned()),
            uname: Cow::from(self.uname.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rattach {
    pub qid: Qid,
}

#[derive(Clone, Debug)]
pub struct Tstatfs {
    pub fid: u32,
}

#[derive(Clone, Debug)]
pub struct Rstatfs {
    pub statfs: Statfs,
}

#[derive(Clone, Debug)]
pub struct Tlopen {
    pub fid: u32,
    pub flags: LOpenFlags,
}

#[derive(Clone, Debug)]
pub struct Rlopen {
    pub qid: Qid,
    pub iounit: u32,
}

#[derive(Clone, Debug)]
pub struct Tlcreate<'a> {
    pub fid: u32,
    pub name: Cow<'a, str>,
    pub flags: LOpenFlags,
    pub mode: u32,
    pub gid: u32,
}

impl<'a> Tlcreate<'a> {
    pub fn clone_static(&'a self) -> Tlcreate<'static> {
        Tlcreate {
            fid: self.fid,
            flags: self.flags,
            gid: self.gid,
            mode: self.mode,
            name: Cow::from(self.name.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rlcreate {
    pub qid: Qid,
    pub iounit: u32,
}

#[derive(Clone, Debug)]
pub struct Tsymlink<'a> {
    pub fid: u32,
    pub name: Cow<'a, str>,
    pub symtgt: Cow<'a, str>,
    pub gid: u32,
}

impl<'a> Tsymlink<'a> {
    pub fn clone_static(&'a self) -> Tsymlink<'static> {
        Tsymlink {
            fid: self.fid,
            name: Cow::from(self.name.clone().into_owned()),
            symtgt: Cow::from(self.symtgt.clone().into_owned()),
            gid: self.gid,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rsymlink {
    pub qid: Qid,
}

#[derive(Clone, Debug)]
pub struct Tmknod<'a> {
    pub dfid: u32,
    pub name: Cow<'a, str>,
    pub mode: u32,
    pub major: u32,
    pub minor: u32,
    pub gid: u32,
}

impl<'a> Tmknod<'a> {
    pub fn clone_static(&'a self) -> Tmknod<'static> {
        Tmknod {
            dfid: self.dfid,
            gid: self.gid,
            major: self.major,
            minor: self.minor,
            mode: self.mode,
            name: Cow::from(self.name.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rmknod {
    pub qid: Qid,
}

#[derive(Clone, Debug)]
pub struct Trename<'a> {
    pub fid: u32,
    pub dfid: u32,
    pub name: Cow<'a, str>,
}

impl<'a> Trename<'a> {
    pub fn clone_static(&'a self) -> Trename<'static> {
        Trename {
            fid: self.fid,
            dfid: self.dfid,
            name: Cow::from(self.name.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rrename {}

#[derive(Clone, Debug)]
pub struct Treadlink {
    pub fid: u32,
}

#[derive(Clone, Debug)]
pub struct Rreadlink<'a> {
    pub target: Cow<'a, str>,
}

impl<'a> Rreadlink<'a> {
    pub fn clone_static(&'a self) -> Rreadlink<'static> {
        Rreadlink {
            target: Cow::from(self.target.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tgetattr {
    pub fid: u32,
    pub req_mask: GetattrMask,
}

#[derive(Clone, Debug)]
pub struct Rgetattr {
    pub valid: GetattrMask,
    pub qid: Qid,
    pub stat: Stat,
}

#[derive(Clone, Debug)]
pub struct Tsetattr {
    pub fid: u32,
    pub valid: SetattrMask,
    pub stat: SetAttr,
}

#[derive(Clone, Debug)]
pub struct Rsetattr {}

#[derive(Clone, Debug)]
pub struct Txattrwalk<'a> {
    pub fid: u32,
    pub new_fid: u32,
    pub name: Cow<'a, str>,
}

impl<'a> Txattrwalk<'a> {
    pub fn clone_static(&'a self) -> Txattrwalk<'static> {
        Txattrwalk {
            fid: self.fid,
            new_fid: self.new_fid,
            name: Cow::from(self.name.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rxattrwalk {
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct Txattrcreate<'a> {
    pub fid: u32,
    pub name: Cow<'a, str>,
    pub attr_size: u64,
    pub flags: u32,
}

impl<'a> Txattrcreate<'a> {
    pub fn clone_static(&'a self) -> Txattrcreate<'static> {
        Txattrcreate {
            fid: self.fid,
            name: Cow::from(self.name.clone().into_owned()),
            attr_size: self.attr_size,
            flags: self.flags,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rxattrcreate {}

#[derive(Clone, Debug)]
pub struct Treaddir {
    pub fid: u32,
    pub offset: u64,
    pub count: u32,
}

#[derive(Clone, Debug)]
pub struct Rreaddir<'a> {
    pub data: DirEntryData<'a>,
}

impl<'a> Rreaddir<'a> {
    pub fn clone_static(&'a self) -> Rreaddir<'static> {
        Rreaddir {
            data: DirEntryData {
                data: self
                    .data
                    .data
                    .iter()
                    .map(|de| DirEntry {
                        qid: de.qid,
                        offset: de.offset,
                        typ: de.typ,
                        name: Cow::from(de.name.clone().into_owned()),
                    })
                    .collect(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tfsync {
    pub fid: u32,
}

#[derive(Clone, Debug)]
pub struct Rfsync {}

#[derive(Clone, Debug)]
pub struct Tlock<'a> {
    pub fid: u32,
    pub flock: Flock<'a>,
}

impl<'a> Tlock<'a> {
    pub fn clone_static(&'a self) -> Tlock<'static> {
        Tlock {
            fid: self.fid,
            flock: Flock {
                typ: self.flock.typ,
                flags: self.flock.flags,
                start: self.flock.start,
                length: self.flock.length,
                proc_id: self.flock.proc_id,
                client_id: Cow::from(self.flock.client_id.clone().into_owned()),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rlock {
    pub status: LockStatus,
}

#[derive(Clone, Debug)]
pub struct Tgetlock<'a> {
    pub fid: u32,
    pub flock: Getlock<'a>,
}

impl<'a> Tgetlock<'a> {
    pub fn clone_static(&'a self) -> Tgetlock<'static> {
        Tgetlock {
            fid: self.fid,
            flock: Getlock {
                typ: self.flock.typ,
                start: self.flock.start,
                length: self.flock.length,
                proc_id: self.flock.proc_id,
                client_id: Cow::from(self.flock.client_id.clone().into_owned()),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rgetlock<'a> {
    pub flock: Getlock<'a>,
}

impl<'a> Rgetlock<'a> {
    pub fn clone_static(&'a self) -> Rgetlock<'static> {
        Rgetlock {
            flock: Getlock {
                typ: self.flock.typ,
                start: self.flock.start,
                length: self.flock.length,
                proc_id: self.flock.proc_id,
                client_id: Cow::from(self.flock.client_id.clone().into_owned()),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tlink<'a> {
    pub dfid: u32,
    pub fid: u32,
    pub name: Cow<'a, str>,
}

impl<'a> Tlink<'a> {
    pub fn clone_static(&'a self) -> Tlink<'static> {
        Tlink {
            fid: self.fid,
            dfid: self.dfid,
            name: Cow::from(self.name.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rlink {}

#[derive(Clone, Debug)]
pub struct Tmkdir<'a> {
    pub dfid: u32,
    pub name: Cow<'a, str>,
    pub mode: u32,
    pub gid: u32,
}

impl<'a> Tmkdir<'a> {
    pub fn clone_static(&'a self) -> Tmkdir<'static> {
        Tmkdir {
            dfid: self.dfid,
            gid: self.gid,
            mode: self.mode,
            name: Cow::from(self.name.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rmkdir {
    pub qid: Qid,
}

#[derive(Clone, Debug)]
pub struct Trenameat<'a> {
    pub olddfid: u32,
    pub oldname: Cow<'a, str>,
    pub newdfid: u32,
    pub newname: Cow<'a, str>,
}

impl<'a> Trenameat<'a> {
    pub fn clone_static(&'a self) -> Trenameat<'static> {
        Trenameat {
            newdfid: self.newdfid,
            olddfid: self.olddfid,
            newname: Cow::from(self.newname.clone().into_owned()),
            oldname: Cow::from(self.oldname.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rrenameat {}

#[derive(Clone, Debug)]
pub struct Tunlinkat<'a> {
    pub dfid: u32,
    pub name: Cow<'a, str>,
    pub flags: u32,
}

impl<'a> Tunlinkat<'a> {
    pub fn clone_static(&'a self) -> Tunlinkat<'static> {
        Tunlinkat {
            dfid: self.dfid,
            flags: self.flags,
            name: Cow::from(self.name.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Runlinkat {}

#[derive(Clone, Debug)]
pub struct Tauth<'a> {
    pub afid: u32,
    pub uname: Cow<'a, str>,
    pub aname: Cow<'a, str>,
    pub n_uname: u32,
}

impl<'a> Tauth<'a> {
    pub fn clone_static(&'a self) -> Tauth<'static> {
        Tauth {
            afid: self.afid,
            n_uname: self.n_uname,
            aname: Cow::from(self.aname.clone().into_owned()),
            uname: Cow::from(self.uname.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rauth {
    pub aqid: Qid,
}

#[derive(Clone, Debug)]
pub struct Tversion<'a> {
    pub msize: u32,
    pub version: Cow<'a, str>,
}

impl<'a> Tversion<'a> {
    pub fn clone_static(&'a self) -> Tversion<'static> {
        Tversion {
            msize: self.msize,
            version: Cow::from(self.version.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rversion<'a> {
    pub msize: u32,
    pub version: Cow<'a, str>,
}

impl<'a> Rversion<'a> {
    pub fn clone_static(&'a self) -> Rversion<'static> {
        Rversion {
            msize: self.msize,
            version: Cow::from(self.version.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tflush {
    pub oldtag: u16,
}

#[derive(Clone, Debug)]
pub struct Rflush {}

#[derive(Clone, Debug)]
pub struct Twalk<'a> {
    pub fid: u32,
    pub new_fid: u32,
    pub wnames: Vec<Cow<'a, str>>,
}

impl<'a> Twalk<'a> {
    pub fn clone_static(&'a self) -> Twalk<'static> {
        Twalk {
            fid: self.fid,
            new_fid: self.new_fid,
            wnames: self
                .wnames
                .iter()
                .map(|n| Cow::from(n.clone().into_owned()))
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rwalk {
    pub wqids: Vec<Qid>,
}
#[derive(Clone, Debug)]
pub struct Tread {
    pub fid: u32,
    pub offset: u64,
    pub count: u32,
}

#[derive(Clone, Debug)]
pub struct Rread<'a> {
    pub data: Cow<'a, [u8]>,
}

impl<'a> Rread<'a> {
    pub fn clone_static(&'a self) -> Rread<'static> {
        Rread {
            data: Cow::from(self.data.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Twrite<'a> {
    pub fid: u32,
    pub offset: u64,
    pub data: Cow<'a, [u8]>,
}

impl<'a> Twrite<'a> {
    pub fn clone_static(&'a self) -> Twrite<'static> {
        Twrite {
            fid: self.fid,
            offset: self.offset,
            data: Cow::from(self.data.clone().into_owned()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rwrite {
    pub count: u32,
}
#[derive(Clone, Debug)]
pub struct Tclunk {
    pub fid: u32,
}
#[derive(Clone, Debug)]
pub struct Rclunk {}

#[derive(Clone, Debug)]
pub struct Tremove {
    pub fid: u32,
}

#[derive(Clone, Debug)]
pub struct Rremove {}

impl<'a> From<Rlerror> for Fcall<'a> {
    fn from(v: Rlerror) -> Fcall<'a> {
        Fcall::Rlerror(v)
    }
}
impl<'a> From<Tattach<'a>> for Fcall<'a> {
    fn from(v: Tattach<'a>) -> Fcall<'a> {
        Fcall::Tattach(v)
    }
}

impl<'a> From<Rattach> for Fcall<'a> {
    fn from(v: Rattach) -> Fcall<'a> {
        Fcall::Rattach(v)
    }
}
impl<'a> From<Tstatfs> for Fcall<'a> {
    fn from(v: Tstatfs) -> Fcall<'a> {
        Fcall::Tstatfs(v)
    }
}
impl<'a> From<Rstatfs> for Fcall<'a> {
    fn from(v: Rstatfs) -> Fcall<'a> {
        Fcall::Rstatfs(v)
    }
}
impl<'a> From<Tlopen> for Fcall<'a> {
    fn from(v: Tlopen) -> Fcall<'a> {
        Fcall::Tlopen(v)
    }
}
impl<'a> From<Rlopen> for Fcall<'a> {
    fn from(v: Rlopen) -> Fcall<'a> {
        Fcall::Rlopen(v)
    }
}
impl<'a> From<Tlcreate<'a>> for Fcall<'a> {
    fn from(v: Tlcreate<'a>) -> Fcall<'a> {
        Fcall::Tlcreate(v)
    }
}
impl<'a> From<Rlcreate> for Fcall<'a> {
    fn from(v: Rlcreate) -> Fcall<'a> {
        Fcall::Rlcreate(v)
    }
}
impl<'a> From<Tsymlink<'a>> for Fcall<'a> {
    fn from(v: Tsymlink<'a>) -> Fcall<'a> {
        Fcall::Tsymlink(v)
    }
}
impl<'a> From<Rsymlink> for Fcall<'a> {
    fn from(v: Rsymlink) -> Fcall<'a> {
        Fcall::Rsymlink(v)
    }
}
impl<'a> From<Tmknod<'a>> for Fcall<'a> {
    fn from(v: Tmknod<'a>) -> Fcall<'a> {
        Fcall::Tmknod(v)
    }
}
impl<'a> From<Rmknod> for Fcall<'a> {
    fn from(v: Rmknod) -> Fcall<'a> {
        Fcall::Rmknod(v)
    }
}
impl<'a> From<Trename<'a>> for Fcall<'a> {
    fn from(v: Trename<'a>) -> Fcall<'a> {
        Fcall::Trename(v)
    }
}
impl<'a> From<Rrename> for Fcall<'a> {
    fn from(v: Rrename) -> Fcall<'a> {
        Fcall::Rrename(v)
    }
}
impl<'a> From<Treadlink> for Fcall<'a> {
    fn from(v: Treadlink) -> Fcall<'a> {
        Fcall::Treadlink(v)
    }
}
impl<'a> From<Rreadlink<'a>> for Fcall<'a> {
    fn from(v: Rreadlink<'a>) -> Fcall<'a> {
        Fcall::Rreadlink(v)
    }
}
impl<'a> From<Tgetattr> for Fcall<'a> {
    fn from(v: Tgetattr) -> Fcall<'a> {
        Fcall::Tgetattr(v)
    }
}
impl<'a> From<Rgetattr> for Fcall<'a> {
    fn from(v: Rgetattr) -> Fcall<'a> {
        Fcall::Rgetattr(v)
    }
}
impl<'a> From<Tsetattr> for Fcall<'a> {
    fn from(v: Tsetattr) -> Fcall<'a> {
        Fcall::Tsetattr(v)
    }
}
impl<'a> From<Rsetattr> for Fcall<'a> {
    fn from(v: Rsetattr) -> Fcall<'a> {
        Fcall::Rsetattr(v)
    }
}
impl<'a> From<Txattrwalk<'a>> for Fcall<'a> {
    fn from(v: Txattrwalk<'a>) -> Fcall<'a> {
        Fcall::Txattrwalk(v)
    }
}
impl<'a> From<Rxattrwalk> for Fcall<'a> {
    fn from(v: Rxattrwalk) -> Fcall<'a> {
        Fcall::Rxattrwalk(v)
    }
}
impl<'a> From<Txattrcreate<'a>> for Fcall<'a> {
    fn from(v: Txattrcreate<'a>) -> Fcall<'a> {
        Fcall::Txattrcreate(v)
    }
}
impl<'a> From<Rxattrcreate> for Fcall<'a> {
    fn from(v: Rxattrcreate) -> Fcall<'a> {
        Fcall::Rxattrcreate(v)
    }
}
impl<'a> From<Treaddir> for Fcall<'a> {
    fn from(v: Treaddir) -> Fcall<'a> {
        Fcall::Treaddir(v)
    }
}
impl<'a> From<Rreaddir<'a>> for Fcall<'a> {
    fn from(v: Rreaddir<'a>) -> Fcall<'a> {
        Fcall::Rreaddir(v)
    }
}
impl<'a> From<Tfsync> for Fcall<'a> {
    fn from(v: Tfsync) -> Fcall<'a> {
        Fcall::Tfsync(v)
    }
}
impl<'a> From<Rfsync> for Fcall<'a> {
    fn from(v: Rfsync) -> Fcall<'a> {
        Fcall::Rfsync(v)
    }
}
impl<'a> From<Tlock<'a>> for Fcall<'a> {
    fn from(v: Tlock<'a>) -> Fcall<'a> {
        Fcall::Tlock(v)
    }
}
impl<'a> From<Rlock> for Fcall<'a> {
    fn from(v: Rlock) -> Fcall<'a> {
        Fcall::Rlock(v)
    }
}
impl<'a> From<Tgetlock<'a>> for Fcall<'a> {
    fn from(v: Tgetlock<'a>) -> Fcall<'a> {
        Fcall::Tgetlock(v)
    }
}
impl<'a> From<Rgetlock<'a>> for Fcall<'a> {
    fn from(v: Rgetlock<'a>) -> Fcall<'a> {
        Fcall::Rgetlock(v)
    }
}
impl<'a> From<Tlink<'a>> for Fcall<'a> {
    fn from(v: Tlink<'a>) -> Fcall<'a> {
        Fcall::Tlink(v)
    }
}
impl<'a> From<Rlink> for Fcall<'a> {
    fn from(v: Rlink) -> Fcall<'a> {
        Fcall::Rlink(v)
    }
}
impl<'a> From<Tmkdir<'a>> for Fcall<'a> {
    fn from(v: Tmkdir<'a>) -> Fcall<'a> {
        Fcall::Tmkdir(v)
    }
}
impl<'a> From<Rmkdir> for Fcall<'a> {
    fn from(v: Rmkdir) -> Fcall<'a> {
        Fcall::Rmkdir(v)
    }
}
impl<'a> From<Trenameat<'a>> for Fcall<'a> {
    fn from(v: Trenameat<'a>) -> Fcall<'a> {
        Fcall::Trenameat(v)
    }
}
impl<'a> From<Rrenameat> for Fcall<'a> {
    fn from(v: Rrenameat) -> Fcall<'a> {
        Fcall::Rrenameat(v)
    }
}
impl<'a> From<Tunlinkat<'a>> for Fcall<'a> {
    fn from(v: Tunlinkat<'a>) -> Fcall<'a> {
        Fcall::Tunlinkat(v)
    }
}
impl<'a> From<Runlinkat> for Fcall<'a> {
    fn from(v: Runlinkat) -> Fcall<'a> {
        Fcall::Runlinkat(v)
    }
}
impl<'a> From<Tauth<'a>> for Fcall<'a> {
    fn from(v: Tauth<'a>) -> Fcall<'a> {
        Fcall::Tauth(v)
    }
}
impl<'a> From<Rauth> for Fcall<'a> {
    fn from(v: Rauth) -> Fcall<'a> {
        Fcall::Rauth(v)
    }
}
impl<'a> From<Tversion<'a>> for Fcall<'a> {
    fn from(v: Tversion<'a>) -> Fcall<'a> {
        Fcall::Tversion(v)
    }
}
impl<'a> From<Rversion<'a>> for Fcall<'a> {
    fn from(v: Rversion<'a>) -> Fcall<'a> {
        Fcall::Rversion(v)
    }
}
impl<'a> From<Tflush> for Fcall<'a> {
    fn from(v: Tflush) -> Fcall<'a> {
        Fcall::Tflush(v)
    }
}
impl<'a> From<Rflush> for Fcall<'a> {
    fn from(v: Rflush) -> Fcall<'a> {
        Fcall::Rflush(v)
    }
}
impl<'a> From<Twalk<'a>> for Fcall<'a> {
    fn from(v: Twalk<'a>) -> Fcall<'a> {
        Fcall::Twalk(v)
    }
}
impl<'a> From<Rwalk> for Fcall<'a> {
    fn from(v: Rwalk) -> Fcall<'a> {
        Fcall::Rwalk(v)
    }
}
impl<'a> From<Tread> for Fcall<'a> {
    fn from(v: Tread) -> Fcall<'a> {
        Fcall::Tread(v)
    }
}
impl<'a> From<Rread<'a>> for Fcall<'a> {
    fn from(v: Rread<'a>) -> Fcall<'a> {
        Fcall::Rread(v)
    }
}
impl<'a> From<Twrite<'a>> for Fcall<'a> {
    fn from(v: Twrite<'a>) -> Fcall<'a> {
        Fcall::Twrite(v)
    }
}
impl<'a> From<Rwrite> for Fcall<'a> {
    fn from(v: Rwrite) -> Fcall<'a> {
        Fcall::Rwrite(v)
    }
}
impl<'a> From<Tclunk> for Fcall<'a> {
    fn from(v: Tclunk) -> Fcall<'a> {
        Fcall::Tclunk(v)
    }
}
impl<'a> From<Rclunk> for Fcall<'a> {
    fn from(v: Rclunk) -> Fcall<'a> {
        Fcall::Rclunk(v)
    }
}
impl<'a> From<Tremove> for Fcall<'a> {
    fn from(v: Tremove) -> Fcall<'a> {
        Fcall::Tremove(v)
    }
}
impl<'a> From<Rremove> for Fcall<'a> {
    fn from(v: Rremove) -> Fcall<'a> {
        Fcall::Rremove(v)
    }
}

#[derive(Clone, Debug)]
pub enum Fcall<'a> {
    Rlerror(Rlerror),
    Tattach(Tattach<'a>),
    Rattach(Rattach),
    Tstatfs(Tstatfs),
    Rstatfs(Rstatfs),
    Tlopen(Tlopen),
    Rlopen(Rlopen),
    Tlcreate(Tlcreate<'a>),
    Rlcreate(Rlcreate),
    Tsymlink(Tsymlink<'a>),
    Rsymlink(Rsymlink),
    Tmknod(Tmknod<'a>),
    Rmknod(Rmknod),
    Trename(Trename<'a>),
    Rrename(Rrename),
    Treadlink(Treadlink),
    Rreadlink(Rreadlink<'a>),
    Tgetattr(Tgetattr),
    Rgetattr(Rgetattr),
    Tsetattr(Tsetattr),
    Rsetattr(Rsetattr),
    Txattrwalk(Txattrwalk<'a>),
    Rxattrwalk(Rxattrwalk),
    Txattrcreate(Txattrcreate<'a>),
    Rxattrcreate(Rxattrcreate),
    Treaddir(Treaddir),
    Rreaddir(Rreaddir<'a>),
    Tfsync(Tfsync),
    Rfsync(Rfsync),
    Tlock(Tlock<'a>),
    Rlock(Rlock),
    Tgetlock(Tgetlock<'a>),
    Rgetlock(Rgetlock<'a>),
    Tlink(Tlink<'a>),
    Rlink(Rlink),
    Tmkdir(Tmkdir<'a>),
    Rmkdir(Rmkdir),
    Trenameat(Trenameat<'a>),
    Rrenameat(Rrenameat),
    Tunlinkat(Tunlinkat<'a>),
    Runlinkat(Runlinkat),
    Tauth(Tauth<'a>),
    Rauth(Rauth),
    Tversion(Tversion<'a>),
    Rversion(Rversion<'a>),
    Tflush(Tflush),
    Rflush(Rflush),
    Twalk(Twalk<'a>),
    Rwalk(Rwalk),
    Tread(Tread),
    Rread(Rread<'a>),
    Twrite(Twrite<'a>),
    Rwrite(Rwrite),
    Tclunk(Tclunk),
    Rclunk(Rclunk),
    Tremove(Tremove),
    Rremove(Rremove),
}

impl<'a> Fcall<'a> {
    pub fn clone_static(&self) -> Fcall<'static> {
        match self {
            Fcall::Rlerror(ref v) => Fcall::Rlerror(v.clone()),
            Fcall::Tattach(ref v) => Fcall::Tattach(v.clone_static()),
            Fcall::Rattach(ref v) => Fcall::Rattach(v.clone()),
            Fcall::Tstatfs(v) => Fcall::Tstatfs(v.clone()),
            Fcall::Rstatfs(v) => Fcall::Rstatfs(v.clone()),
            Fcall::Tlopen(v) => Fcall::Tlopen(v.clone()),
            Fcall::Rlopen(v) => Fcall::Rlopen(v.clone()),
            Fcall::Tlcreate(v) => Fcall::Tlcreate(v.clone_static()),
            Fcall::Rlcreate(v) => Fcall::Rlcreate(v.clone()),
            Fcall::Tsymlink(v) => Fcall::Tsymlink(v.clone_static()),
            Fcall::Rsymlink(v) => Fcall::Rsymlink(v.clone()),
            Fcall::Tmknod(v) => Fcall::Tmknod(v.clone_static()),
            Fcall::Rmknod(v) => Fcall::Rmknod(v.clone()),
            Fcall::Trename(v) => Fcall::Trename(v.clone_static()),
            Fcall::Rrename(v) => Fcall::Rrename(v.clone()),
            Fcall::Treadlink(v) => Fcall::Treadlink(v.clone()),
            Fcall::Rreadlink(v) => Fcall::Rreadlink(v.clone_static()),
            Fcall::Tgetattr(v) => Fcall::Tgetattr(v.clone()),
            Fcall::Rgetattr(v) => Fcall::Rgetattr(v.clone()),
            Fcall::Tsetattr(v) => Fcall::Tsetattr(v.clone()),
            Fcall::Rsetattr(v) => Fcall::Rsetattr(v.clone()),
            Fcall::Txattrwalk(v) => Fcall::Txattrwalk(v.clone_static()),
            Fcall::Rxattrwalk(v) => Fcall::Rxattrwalk(v.clone()),
            Fcall::Txattrcreate(v) => Fcall::Txattrcreate(v.clone_static()),
            Fcall::Rxattrcreate(v) => Fcall::Rxattrcreate(v.clone()),
            Fcall::Treaddir(v) => Fcall::Treaddir(v.clone()),
            Fcall::Rreaddir(v) => Fcall::Rreaddir(v.clone_static()),
            Fcall::Tfsync(v) => Fcall::Tfsync(v.clone()),
            Fcall::Rfsync(v) => Fcall::Rfsync(v.clone()),
            Fcall::Tlock(v) => Fcall::Tlock(v.clone_static()),
            Fcall::Rlock(v) => Fcall::Rlock(v.clone()),
            Fcall::Tgetlock(v) => Fcall::Tgetlock(v.clone_static()),
            Fcall::Rgetlock(v) => Fcall::Rgetlock(v.clone_static()),
            Fcall::Tlink(v) => Fcall::Tlink(v.clone_static()),
            Fcall::Rlink(v) => Fcall::Rlink(v.clone()),
            Fcall::Tmkdir(v) => Fcall::Tmkdir(v.clone_static()),
            Fcall::Rmkdir(v) => Fcall::Rmkdir(v.clone()),
            Fcall::Trenameat(v) => Fcall::Trenameat(v.clone_static()),
            Fcall::Rrenameat(v) => Fcall::Rrenameat(v.clone()),
            Fcall::Tunlinkat(v) => Fcall::Tunlinkat(v.clone_static()),
            Fcall::Runlinkat(v) => Fcall::Runlinkat(v.clone()),
            Fcall::Tauth(v) => Fcall::Tauth(v.clone_static()),
            Fcall::Rauth(v) => Fcall::Rauth(v.clone()),
            Fcall::Tversion(v) => Fcall::Tversion(v.clone_static()),
            Fcall::Rversion(v) => Fcall::Rversion(v.clone_static()),
            Fcall::Tflush(v) => Fcall::Tflush(v.clone()),
            Fcall::Rflush(v) => Fcall::Rflush(v.clone()),
            Fcall::Twalk(v) => Fcall::Twalk(v.clone_static()),
            Fcall::Rwalk(v) => Fcall::Rwalk(v.clone()),
            Fcall::Tread(v) => Fcall::Tread(v.clone()),
            Fcall::Rread(v) => Fcall::Rread(v.clone_static()),
            Fcall::Twrite(v) => Fcall::Twrite(v.clone_static()),
            Fcall::Rwrite(v) => Fcall::Rwrite(v.clone()),
            Fcall::Tclunk(v) => Fcall::Tclunk(v.clone()),
            Fcall::Rclunk(v) => Fcall::Rclunk(v.clone()),
            Fcall::Tremove(v) => Fcall::Tremove(v.clone()),
            Fcall::Rremove(v) => Fcall::Rremove(v.clone()),
        }
    }
}

impl<'a, 'b> From<&'a Fcall<'b>> for FcallType {
    fn from(fcall: &'a Fcall<'b>) -> FcallType {
        match *fcall {
            Fcall::Rlerror(_) => FcallType::Rlerror,
            Fcall::Tattach(_) => FcallType::Tattach,
            Fcall::Rattach(_) => FcallType::Rattach,
            Fcall::Tstatfs(_) => FcallType::Tstatfs,
            Fcall::Rstatfs(_) => FcallType::Rstatfs,
            Fcall::Tlopen(_) => FcallType::Tlopen,
            Fcall::Rlopen(_) => FcallType::Rlopen,
            Fcall::Tlcreate(_) => FcallType::Tlcreate,
            Fcall::Rlcreate(_) => FcallType::Rlcreate,
            Fcall::Tsymlink(_) => FcallType::Tsymlink,
            Fcall::Rsymlink(_) => FcallType::Rsymlink,
            Fcall::Tmknod(_) => FcallType::Tmknod,
            Fcall::Rmknod(_) => FcallType::Rmknod,
            Fcall::Trename(_) => FcallType::Trename,
            Fcall::Rrename(_) => FcallType::Rrename,
            Fcall::Treadlink(_) => FcallType::Treadlink,
            Fcall::Rreadlink(_) => FcallType::Rreadlink,
            Fcall::Tgetattr(_) => FcallType::Tgetattr,
            Fcall::Rgetattr(_) => FcallType::Rgetattr,
            Fcall::Tsetattr(_) => FcallType::Tsetattr,
            Fcall::Rsetattr(_) => FcallType::Rsetattr,
            Fcall::Txattrwalk(_) => FcallType::Txattrwalk,
            Fcall::Rxattrwalk(_) => FcallType::Rxattrwalk,
            Fcall::Txattrcreate(_) => FcallType::Txattrcreate,
            Fcall::Rxattrcreate(_) => FcallType::Rxattrcreate,
            Fcall::Treaddir(_) => FcallType::Treaddir,
            Fcall::Rreaddir(_) => FcallType::Rreaddir,
            Fcall::Tfsync(_) => FcallType::Tfsync,
            Fcall::Rfsync(_) => FcallType::Rfsync,
            Fcall::Tlock(_) => FcallType::Tlock,
            Fcall::Rlock(_) => FcallType::Rlock,
            Fcall::Tgetlock(_) => FcallType::Tgetlock,
            Fcall::Rgetlock(_) => FcallType::Rgetlock,
            Fcall::Tlink(_) => FcallType::Tlink,
            Fcall::Rlink(_) => FcallType::Rlink,
            Fcall::Tmkdir(_) => FcallType::Tmkdir,
            Fcall::Rmkdir(_) => FcallType::Rmkdir,
            Fcall::Trenameat(_) => FcallType::Trenameat,
            Fcall::Rrenameat(_) => FcallType::Rrenameat,
            Fcall::Tunlinkat(_) => FcallType::Tunlinkat,
            Fcall::Runlinkat(_) => FcallType::Runlinkat,
            Fcall::Tauth(_) => FcallType::Tauth,
            Fcall::Rauth(_) => FcallType::Rauth,
            Fcall::Tversion(_) => FcallType::Tversion,
            Fcall::Rversion(_) => FcallType::Rversion,
            Fcall::Tflush(_) => FcallType::Tflush,
            Fcall::Rflush(_) => FcallType::Rflush,
            Fcall::Twalk(_) => FcallType::Twalk,
            Fcall::Rwalk(_) => FcallType::Rwalk,
            Fcall::Tread(_) => FcallType::Tread,
            Fcall::Rread(_) => FcallType::Rread,
            Fcall::Twrite(_) => FcallType::Twrite,
            Fcall::Rwrite(_) => FcallType::Rwrite,
            Fcall::Tclunk(_) => FcallType::Tclunk,
            Fcall::Rclunk(_) => FcallType::Rclunk,
            Fcall::Tremove(_) => FcallType::Tremove,
            Fcall::Rremove(_) => FcallType::Rremove,
        }
    }
}

impl<'a, A: Into<Fcall<'a>>, B: Into<Fcall<'a>>> From<Result<A, B>> for Fcall<'a> {
    fn from(r: Result<A, B>) -> Fcall<'a> {
        match r {
            Ok(a) => a.into(),
            Err(b) => b.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaggedFcall<'a> {
    pub tag: u16,
    pub fcall: Fcall<'a>,
}

impl<'a> TaggedFcall<'a> {
    pub fn clone_static(&self) -> TaggedFcall<'static> {
        TaggedFcall {
            tag: self.tag,
            fcall: self.fcall.clone_static(),
        }
    }

    pub fn encode_to_buf(&self, buf: &mut Vec<u8>) -> std::io::Result<()> {
        buf.truncate(0);
        let mut w = std::io::Cursor::new(buf);
        w.write_all(&[0, 0, 0, 0])?;
        let typ = FcallType::from(&self.fcall);
        encode_u8(&mut w, typ as u8)?;
        encode_u16(&mut w, self.tag)?;
        match self.fcall {
            Fcall::Rlerror(ref v) => encode_rlerror(&mut w, v)?,
            Fcall::Tattach(ref v) => encode_tattach(&mut w, v)?,
            Fcall::Rattach(ref v) => encode_rattach(&mut w, v)?,
            Fcall::Tstatfs(ref v) => encode_tstatfs(&mut w, v)?,
            Fcall::Rstatfs(ref v) => encode_rstatfs(&mut w, v)?,
            Fcall::Tlopen(ref v) => encode_tlopen(&mut w, v)?,
            Fcall::Rlopen(ref v) => encode_rlopen(&mut w, v)?,
            Fcall::Tlcreate(ref v) => encode_tlcreate(&mut w, v)?,
            Fcall::Rlcreate(ref v) => encode_rlcreate(&mut w, v)?,
            Fcall::Tsymlink(ref v) => encode_tsymlink(&mut w, v)?,
            Fcall::Rsymlink(ref v) => encode_rsymlink(&mut w, v)?,
            Fcall::Tmknod(ref v) => encode_tmknod(&mut w, v)?,
            Fcall::Rmknod(ref v) => encode_rmknod(&mut w, v)?,
            Fcall::Trename(ref v) => encode_trename(&mut w, v)?,
            Fcall::Rrename(ref v) => encode_rrename(&mut w, v)?,
            Fcall::Treadlink(ref v) => encode_treadlink(&mut w, v)?,
            Fcall::Rreadlink(ref v) => encode_rreadlink(&mut w, v)?,
            Fcall::Tgetattr(ref v) => encode_tgetattr(&mut w, v)?,
            Fcall::Rgetattr(ref v) => encode_rgetattr(&mut w, v)?,
            Fcall::Tsetattr(ref v) => encode_tsetattr(&mut w, v)?,
            Fcall::Rsetattr(ref v) => encode_rsetattr(&mut w, v)?,
            Fcall::Txattrwalk(ref v) => encode_txattrwalk(&mut w, v)?,
            Fcall::Rxattrwalk(ref v) => encode_rxattrwalk(&mut w, v)?,
            Fcall::Txattrcreate(ref v) => encode_txattrcreate(&mut w, v)?,
            Fcall::Rxattrcreate(ref v) => encode_rxattrcreate(&mut w, v)?,
            Fcall::Treaddir(ref v) => encode_treaddir(&mut w, v)?,
            Fcall::Rreaddir(ref v) => encode_rreaddir(&mut w, v)?,
            Fcall::Tfsync(ref v) => encode_tfsync(&mut w, v)?,
            Fcall::Rfsync(ref v) => encode_rfsync(&mut w, v)?,
            Fcall::Tlock(ref v) => encode_tlock(&mut w, v)?,
            Fcall::Rlock(ref v) => encode_rlock(&mut w, v)?,
            Fcall::Tgetlock(ref v) => encode_tgetlock(&mut w, v)?,
            Fcall::Rgetlock(ref v) => encode_rgetlock(&mut w, v)?,
            Fcall::Tlink(ref v) => encode_tlink(&mut w, v)?,
            Fcall::Rlink(ref v) => encode_rlink(&mut w, v)?,
            Fcall::Tmkdir(ref v) => encode_tmkdir(&mut w, v)?,
            Fcall::Rmkdir(ref v) => encode_rmkdir(&mut w, v)?,
            Fcall::Trenameat(ref v) => encode_trenameat(&mut w, v)?,
            Fcall::Rrenameat(ref v) => encode_rrenameat(&mut w, v)?,
            Fcall::Tunlinkat(ref v) => encode_tunlinkat(&mut w, v)?,
            Fcall::Runlinkat(ref v) => encode_runlinkat(&mut w, v)?,
            Fcall::Tauth(ref v) => encode_tauth(&mut w, v)?,
            Fcall::Rauth(ref v) => encode_rauth(&mut w, v)?,
            Fcall::Tversion(ref v) => encode_tversion(&mut w, v)?,
            Fcall::Rversion(ref v) => encode_rversion(&mut w, v)?,
            Fcall::Tflush(ref v) => encode_tflush(&mut w, v)?,
            Fcall::Rflush(ref v) => encode_rflush(&mut w, v)?,
            Fcall::Twalk(ref v) => encode_twalk(&mut w, v)?,
            Fcall::Rwalk(ref v) => encode_rwalk(&mut w, v)?,
            Fcall::Tread(ref v) => encode_tread(&mut w, v)?,
            Fcall::Rread(ref v) => encode_rread(&mut w, v)?,
            Fcall::Twrite(ref v) => encode_twrite(&mut w, v)?,
            Fcall::Rwrite(ref v) => encode_rwrite(&mut w, v)?,
            Fcall::Tclunk(ref v) => encode_tclunk(&mut w, v)?,
            Fcall::Rclunk(ref v) => encode_rclunk(&mut w, v)?,
            Fcall::Tremove(ref v) => encode_tremove(&mut w, v)?,
            Fcall::Rremove(ref v) => encode_rremove(&mut w, v)?,
        };
        let buf = w.into_inner();
        let sz_bytes = &(buf.len() as u32).to_le_bytes()[..];
        (&mut buf[..4]).copy_from_slice(sz_bytes);
        Ok(())
    }

    pub fn decode(buf: &'a [u8]) -> Result<TaggedFcall<'a>, std::io::Error> {
        let mut d = FcallDecoder { buf };
        d.decode_u32()?; // Skip size.
        d.decode()
    }
}

pub fn read_to_buf<R: Read>(r: &mut R, buf: &mut Vec<u8>) -> std::io::Result<()> {
    buf.resize(4, 0);
    r.read_exact(&mut buf[..])?;
    let sz = u32::from_le_bytes(buf[..4].try_into().unwrap()) as usize;
    if sz > buf.capacity() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "9p remote violated protocol size limit",
        ));
    }
    buf.resize(sz, 0);
    r.read_exact(&mut buf[4..])?;
    Ok(())
}

// Returns std::io::ErrorKind::TimedOut on timeout.
pub fn read_to_buf_timeout(
    conn: &mut std::net::TcpStream,
    fcall_buf: &mut Vec<u8>,
    timeout: std::time::Duration,
) -> Result<(), std::io::Error> {
    let old_timeout = conn.read_timeout()?;
    conn.set_read_timeout(Some(timeout))?;

    fcall_buf.resize(4, 0);
    let read_result = conn.read(&mut fcall_buf[..4]);

    conn.set_read_timeout(old_timeout)?;

    match read_result {
        Ok(n) => {
            if n < 4 {
                conn.read_exact(&mut fcall_buf[n..4])?;
            }
        }
        Err(err) => match err.kind() {
            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => {
                return Err(std::io::Error::from(std::io::ErrorKind::TimedOut));
            }
            _ => return Err(err),
        },
    };

    let sz = u32::from_le_bytes(fcall_buf[..4].try_into().unwrap()) as usize;
    if sz > fcall_buf.capacity() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "9p remote violated protocol size limit",
        ));
    }

    fcall_buf.resize(sz, 0);
    conn.read_exact(&mut fcall_buf[4..sz])?;

    Ok(())
}

pub fn read<'a, R: Read>(
    r: &mut R,
    buf: &'a mut Vec<u8>,
) -> Result<TaggedFcall<'a>, std::io::Error> {
    read_to_buf(r, buf)?;
    TaggedFcall::decode(&buf[..])
}

pub fn write<W: Write>(w: &mut W, buf: &mut Vec<u8>, fcall: &TaggedFcall) -> std::io::Result<()> {
    buf.truncate(0);
    match fcall {
        TaggedFcall {
            tag,
            fcall: Fcall::Rread(Rread { data }),
        } => {
            // Zero copy Rread path.
            let sz = 4 + 1 + 2 + 4 + data.len();
            if sz > buf.capacity() {
                // The message was larger than the buffer.
                // This must be larger than msize so flag the mistake.
                return Err(invalid_9p_msg());
            }
            let mut cursor = std::io::Cursor::new(buf);
            encode_u32(&mut cursor, sz as u32)?;
            encode_u8(&mut cursor, 117)?;
            encode_u16(&mut cursor, *tag)?;
            encode_u32(&mut cursor, data.len() as u32)?;
            let buf = cursor.into_inner();
            // XXX: Could be a vectored write here?
            w.write_all(&buf[..])?;
            w.write_all(&data[..])?;
            Ok(())
        }
        TaggedFcall {
            tag,
            fcall: Fcall::Twrite(Twrite { fid, offset, data }),
        } => {
            // Zero copy Twrite path.
            let sz = 4 + 1 + 2 + 4 + 8 + 4 + data.len();
            if sz > buf.capacity() {
                // The message was larger than the buffer.
                // This must be larger than msize so flag the mistake.
                return Err(invalid_9p_msg());
            }
            let mut cursor = std::io::Cursor::new(buf);
            encode_u32(&mut cursor, sz as u32)?;
            encode_u8(&mut cursor, 118)?;
            encode_u16(&mut cursor, *tag)?;
            encode_u32(&mut cursor, *fid)?;
            encode_u64(&mut cursor, *offset)?;
            encode_u32(&mut cursor, data.len() as u32)?;
            let buf = cursor.into_inner();
            // XXX: Could be a vectored write here?
            w.write_all(&buf[..])?;
            w.write_all(&data[..])?;
            Ok(())
        }
        fcall => {
            // Slow path, encode the whole message to the buffer then write it.
            fcall.encode_to_buf(buf)?;
            w.write_all(&buf[..])?;
            Ok(())
        }
    }
}

fn encode_u8<W: Write>(w: &mut W, v: u8) -> std::io::Result<()> {
    w.write_all(&[v])?;
    Ok(())
}

fn encode_u16<W: Write>(w: &mut W, v: u16) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes()[..])?;
    Ok(())
}

fn encode_u32<W: Write>(w: &mut W, v: u32) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes()[..])?;
    Ok(())
}

fn encode_u64<W: Write>(w: &mut W, v: u64) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes()[..])?;
    Ok(())
}

fn encode_str<W: Write>(w: &mut W, v: &str) -> std::io::Result<()> {
    if v.len() > 0xffff {
        return Err(std::io::Error::new(
            ::std::io::ErrorKind::InvalidInput,
            "string too long for 9p encoding",
        ));
    }
    encode_u16(w, v.len() as u16)?;
    w.write_all(v.as_bytes())?;
    Ok(())
}

fn encode_data_buf<W: Write>(w: &mut W, v: &[u8]) -> std::io::Result<()> {
    if v.len() > 0xffffffff {
        return Err(std::io::Error::new(
            ::std::io::ErrorKind::InvalidInput,
            "data buf too long for 9p encoding",
        ));
    }
    encode_u32(w, v.len() as u32)?;
    w.write_all(v)?;
    Ok(())
}

fn encode_vec_str<'a, W: Write>(w: &mut W, v: &[Cow<'a, str>]) -> std::io::Result<()> {
    if v.len() > 0xffff {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "string vec too long for 9p encoding",
        ));
    }
    encode_u16(w, v.len() as u16)?;
    for v in v.iter() {
        encode_str(w, v)?;
    }
    Ok(())
}

fn encode_vec_qid<W: Write>(w: &mut W, v: &[Qid]) -> std::io::Result<()> {
    if v.len() > 0xffff {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "qid vec too long for 9p encoding",
        ));
    }
    encode_u16(w, v.len() as u16)?;
    for v in v.iter() {
        encode_qid(w, v)?;
    }
    Ok(())
}

fn encode_direntrydata<'a, 'b, W: Write>(
    w: &'a mut W,
    v: &'a DirEntryData<'b>,
) -> std::io::Result<()> {
    let data_size = v.size();
    if data_size > 0xffffffff {
        return Err(std::io::Error::new(
            ::std::io::ErrorKind::InvalidInput,
            "dir entry vec too long for encoding",
        ));
    }
    encode_u32(w, data_size as u32)?;
    for v in v.data.iter() {
        encode_direntry(w, v)?;
    }
    Ok(())
}

fn encode_qidtype<W: Write>(w: &mut W, v: &QidType) -> std::io::Result<()> {
    encode_u8(w, v.bits())
}

fn encode_locktype<W: Write>(w: &mut W, v: &LockType) -> std::io::Result<()> {
    encode_u8(w, v.bits())
}

fn encode_lockstatus<W: Write>(w: &mut W, v: &LockStatus) -> std::io::Result<()> {
    encode_u8(w, v.bits())
}

fn encode_lockflag<W: Write>(w: &mut W, v: &LockFlag) -> std::io::Result<()> {
    encode_u32(w, v.bits())
}

fn encode_getattrmask<W: Write>(w: &mut W, v: &GetattrMask) -> std::io::Result<()> {
    encode_u64(w, v.bits())
}

fn encode_setattrmask<W: Write>(w: &mut W, v: &SetattrMask) -> std::io::Result<()> {
    encode_u32(w, v.bits())
}

fn encode_qid<W: Write>(w: &mut W, v: &Qid) -> std::io::Result<()> {
    encode_qidtype(w, &v.typ)?;
    encode_u32(w, v.version)?;
    encode_u64(w, v.path)?;
    Ok(())
}

fn encode_statfs<W: Write>(w: &mut W, v: &Statfs) -> std::io::Result<()> {
    encode_u32(w, v.typ)?;
    encode_u32(w, v.bsize)?;
    encode_u64(w, v.blocks)?;
    encode_u64(w, v.bfree)?;
    encode_u64(w, v.bavail)?;
    encode_u64(w, v.files)?;
    encode_u64(w, v.ffree)?;
    encode_u64(w, v.fsid)?;
    encode_u32(w, v.namelen)?;
    Ok(())
}

fn encode_time<W: Write>(w: &mut W, v: &Time) -> std::io::Result<()> {
    encode_u64(w, v.sec)?;
    encode_u64(w, v.nsec)?;
    Ok(())
}

fn encode_stat<W: Write>(w: &mut W, v: &Stat) -> std::io::Result<()> {
    encode_u32(w, v.mode)?;
    encode_u32(w, v.uid)?;
    encode_u32(w, v.gid)?;
    encode_u64(w, v.nlink)?;
    encode_u64(w, v.rdev)?;
    encode_u64(w, v.size)?;
    encode_u64(w, v.blksize)?;
    encode_u64(w, v.blocks)?;
    encode_time(w, &v.atime)?;
    encode_time(w, &v.mtime)?;
    encode_time(w, &v.ctime)?;
    encode_time(w, &v.btime)?;
    encode_u64(w, v.gen)?;
    encode_u64(w, v.data_version)?;
    Ok(())
}

fn encode_setattr<W: Write>(w: &mut W, v: &SetAttr) -> std::io::Result<()> {
    encode_u32(w, v.mode)?;
    encode_u32(w, v.uid)?;
    encode_u32(w, v.gid)?;
    encode_u64(w, v.size)?;
    encode_time(w, &v.atime)?;
    encode_time(w, &v.mtime)?;
    Ok(())
}

fn encode_direntry<'a, W: Write>(w: &mut W, v: &DirEntry<'a>) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    encode_u64(w, v.offset)?;
    encode_u8(w, v.typ)?;
    encode_str(w, &v.name)?;
    Ok(())
}

fn encode_flock<'a, W: Write>(w: &mut W, v: &Flock<'a>) -> std::io::Result<()> {
    encode_locktype(w, &v.typ)?;
    encode_lockflag(w, &v.flags)?;
    encode_u64(w, v.start)?;
    encode_u64(w, v.length)?;
    encode_u32(w, v.proc_id)?;
    encode_str(w, &v.client_id)?;
    Ok(())
}

fn encode_getlock<'a, W: Write>(w: &'a mut W, v: &Getlock<'a>) -> std::io::Result<()> {
    encode_locktype(w, &v.typ)?;
    encode_u64(w, v.start)?;
    encode_u64(w, v.length)?;
    encode_u32(w, v.proc_id)?;
    encode_str(w, &v.client_id)?;
    Ok(())
}

fn encode_rlerror<W: Write>(w: &mut W, v: &Rlerror) -> std::io::Result<()> {
    encode_u32(w, v.ecode)?;
    Ok(())
}
fn encode_tattach<'a, W: Write>(w: &'a mut W, v: &Tattach<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_u32(w, v.afid)?;
    encode_str(w, &v.uname)?;
    encode_str(w, &v.aname)?;
    encode_u32(w, v.n_uname)?;
    Ok(())
}

fn encode_rattach<W: Write>(w: &mut W, v: &Rattach) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    Ok(())
}
fn encode_tstatfs<W: Write>(w: &mut W, v: &Tstatfs) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    Ok(())
}
fn encode_rstatfs<W: Write>(w: &mut W, v: &Rstatfs) -> std::io::Result<()> {
    encode_statfs(w, &v.statfs)?;
    Ok(())
}
fn encode_tlopen<W: Write>(w: &mut W, v: &Tlopen) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_u32(w, v.flags.bits())?;
    Ok(())
}
fn encode_rlopen<W: Write>(w: &mut W, v: &Rlopen) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    encode_u32(w, v.iounit)?;
    Ok(())
}
fn encode_tlcreate<'a, W: Write>(w: &'a mut W, v: &Tlcreate<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_str(w, &v.name)?;
    encode_u32(w, v.flags.bits())?;
    encode_u32(w, v.mode)?;
    encode_u32(w, v.gid)?;
    Ok(())
}

fn encode_rlcreate<W: Write>(w: &mut W, v: &Rlcreate) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    encode_u32(w, v.iounit)?;
    Ok(())
}

fn encode_tsymlink<'a, W: Write>(w: &'a mut W, v: &Tsymlink<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_str(w, &v.name)?;
    encode_str(w, &v.symtgt)?;
    encode_u32(w, v.gid)?;
    Ok(())
}

fn encode_rsymlink<W: Write>(w: &mut W, v: &Rsymlink) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    Ok(())
}

fn encode_tmknod<'a, W: Write>(w: &'a mut W, v: &Tmknod<'a>) -> std::io::Result<()> {
    encode_u32(w, v.dfid)?;
    encode_str(w, &v.name)?;
    encode_u32(w, v.mode)?;
    encode_u32(w, v.major)?;
    encode_u32(w, v.minor)?;
    encode_u32(w, v.gid)?;
    Ok(())
}

fn encode_rmknod<W: Write>(w: &mut W, v: &Rmknod) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    Ok(())
}

fn encode_trename<'a, W: Write>(w: &'a mut W, v: &Trename<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_u32(w, v.dfid)?;
    encode_str(w, &v.name)?;
    Ok(())
}

fn encode_rrename<W: Write>(_w: &mut W, _v: &Rrename) -> std::io::Result<()> {
    Ok(())
}

fn encode_treadlink<W: Write>(w: &mut W, v: &Treadlink) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    Ok(())
}

fn encode_rreadlink<'a, W: Write>(w: &'a mut W, v: &Rreadlink<'a>) -> std::io::Result<()> {
    encode_str(w, &v.target)?;
    Ok(())
}

fn encode_tgetattr<W: Write>(w: &mut W, v: &Tgetattr) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_getattrmask(w, &v.req_mask)?;
    Ok(())
}
fn encode_rgetattr<W: Write>(w: &mut W, v: &Rgetattr) -> std::io::Result<()> {
    encode_getattrmask(w, &v.valid)?;
    encode_qid(w, &v.qid)?;
    encode_stat(w, &v.stat)?;
    Ok(())
}
fn encode_tsetattr<W: Write>(w: &mut W, v: &Tsetattr) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_setattrmask(w, &v.valid)?;
    encode_setattr(w, &v.stat)?;
    Ok(())
}
fn encode_rsetattr<W: Write>(_w: &mut W, _v: &Rsetattr) -> std::io::Result<()> {
    Ok(())
}
fn encode_txattrwalk<'a, W: Write>(w: &mut W, v: &Txattrwalk<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_u32(w, v.new_fid)?;
    encode_str(w, &v.name)?;
    Ok(())
}

fn encode_rxattrwalk<W: Write>(w: &mut W, v: &Rxattrwalk) -> std::io::Result<()> {
    encode_u64(w, v.size)?;
    Ok(())
}

fn encode_txattrcreate<'a, W: Write>(w: &mut W, v: &Txattrcreate<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_str(w, &v.name)?;
    encode_u64(w, v.attr_size)?;
    encode_u32(w, v.flags)?;
    Ok(())
}

fn encode_rxattrcreate<W: Write>(_w: &mut W, _v: &Rxattrcreate) -> std::io::Result<()> {
    Ok(())
}

fn encode_treaddir<W: Write>(w: &mut W, v: &Treaddir) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_u64(w, v.offset)?;
    encode_u32(w, v.count)?;
    Ok(())
}

fn encode_rreaddir<'a, W: Write>(w: &'a mut W, v: &Rreaddir<'a>) -> std::io::Result<()> {
    encode_direntrydata(w, &v.data)?;
    Ok(())
}

fn encode_tfsync<W: Write>(w: &mut W, v: &Tfsync) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    Ok(())
}

fn encode_rfsync<W: Write>(_w: &mut W, _v: &Rfsync) -> std::io::Result<()> {
    Ok(())
}
fn encode_tlock<'a, W: Write>(w: &'a mut W, v: &Tlock<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_flock(w, &v.flock)?;
    Ok(())
}

fn encode_rlock<W: Write>(w: &mut W, v: &Rlock) -> std::io::Result<()> {
    encode_lockstatus(w, &v.status)?;
    Ok(())
}

fn encode_tgetlock<'a, W: Write>(w: &'a mut W, v: &Tgetlock<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_getlock(w, &v.flock)?;
    Ok(())
}

fn encode_rgetlock<'a, W: Write>(w: &'a mut W, v: &Rgetlock<'a>) -> std::io::Result<()> {
    encode_getlock(w, &v.flock)?;
    Ok(())
}

fn encode_tlink<'a, W: Write>(w: &'a mut W, v: &Tlink<'a>) -> std::io::Result<()> {
    encode_u32(w, v.dfid)?;
    encode_u32(w, v.fid)?;
    encode_str(w, &v.name)?;
    Ok(())
}

fn encode_rlink<W: Write>(_w: &mut W, _v: &Rlink) -> std::io::Result<()> {
    Ok(())
}

fn encode_tmkdir<'a, W: Write>(w: &'a mut W, v: &Tmkdir<'a>) -> std::io::Result<()> {
    encode_u32(w, v.dfid)?;
    encode_str(w, &v.name)?;
    encode_u32(w, v.mode)?;
    encode_u32(w, v.gid)?;
    Ok(())
}

fn encode_rmkdir<W: Write>(w: &mut W, v: &Rmkdir) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    Ok(())
}
fn encode_trenameat<'a, W: Write>(w: &'a mut W, v: &Trenameat<'a>) -> std::io::Result<()> {
    encode_u32(w, v.olddfid)?;
    encode_str(w, &v.oldname)?;
    encode_u32(w, v.newdfid)?;
    encode_str(w, &v.newname)?;
    Ok(())
}

fn encode_rrenameat<W: Write>(_w: &mut W, _v: &Rrenameat) -> std::io::Result<()> {
    Ok(())
}

fn encode_tunlinkat<'a, W: Write>(w: &'a mut W, v: &Tunlinkat<'a>) -> std::io::Result<()> {
    encode_u32(w, v.dfid)?;
    encode_str(w, &v.name)?;
    encode_u32(w, v.flags)?;
    Ok(())
}

fn encode_runlinkat<W: Write>(_w: &mut W, _v: &Runlinkat) -> std::io::Result<()> {
    Ok(())
}

fn encode_tauth<'a, W: Write>(w: &'a mut W, v: &Tauth<'a>) -> std::io::Result<()> {
    encode_u32(w, v.afid)?;
    encode_str(w, &v.uname)?;
    encode_str(w, &v.aname)?;
    encode_u32(w, v.n_uname)?;
    Ok(())
}

fn encode_rauth<W: Write>(w: &mut W, v: &Rauth) -> std::io::Result<()> {
    encode_qid(w, &v.aqid)?;
    Ok(())
}

fn encode_tversion<'a, W: Write>(w: &'a mut W, v: &Tversion<'a>) -> std::io::Result<()> {
    encode_u32(w, v.msize)?;
    encode_str(w, &v.version)?;
    Ok(())
}

fn encode_rversion<'a, W: Write>(w: &'a mut W, v: &Rversion<'a>) -> std::io::Result<()> {
    encode_u32(w, v.msize)?;
    encode_str(w, &v.version)?;
    Ok(())
}

fn encode_tflush<W: Write>(w: &mut W, v: &Tflush) -> std::io::Result<()> {
    encode_u16(w, v.oldtag)?;
    Ok(())
}

fn encode_rflush<W: Write>(_w: &mut W, _v: &Rflush) -> std::io::Result<()> {
    Ok(())
}

fn encode_twalk<'a, W: Write>(w: &'a mut W, v: &Twalk<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_u32(w, v.new_fid)?;
    encode_vec_str(w, &v.wnames)?;
    Ok(())
}

fn encode_rwalk<W: Write>(w: &mut W, v: &Rwalk) -> std::io::Result<()> {
    encode_vec_qid(w, &v.wqids)?;
    Ok(())
}

fn encode_tread<W: Write>(w: &mut W, v: &Tread) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_u64(w, v.offset)?;
    encode_u32(w, v.count)?;
    Ok(())
}

fn encode_rread<'a, W: Write>(w: &'a mut W, v: &Rread<'a>) -> std::io::Result<()> {
    encode_data_buf(w, &v.data)?;
    Ok(())
}

fn encode_twrite<'a, W: Write>(w: &'a mut W, v: &Twrite<'a>) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    encode_u64(w, v.offset)?;
    encode_data_buf(w, &v.data)?;
    Ok(())
}

fn encode_rwrite<W: Write>(w: &mut W, v: &Rwrite) -> std::io::Result<()> {
    encode_u32(w, v.count)?;
    Ok(())
}

fn encode_tclunk<W: Write>(w: &mut W, v: &Tclunk) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    Ok(())
}

fn encode_rclunk<W: Write>(_w: &mut W, _v: &Rclunk) -> std::io::Result<()> {
    Ok(())
}

fn encode_tremove<W: Write>(w: &mut W, v: &Tremove) -> std::io::Result<()> {
    encode_u32(w, v.fid)?;
    Ok(())
}

fn encode_rremove<W: Write>(_w: &mut W, _v: &Rremove) -> std::io::Result<()> {
    Ok(())
}

struct FcallDecoder<'b> {
    buf: &'b [u8],
}

fn invalid_9p_msg() -> std::io::Error {
    std::io::Error::new(::std::io::ErrorKind::InvalidInput, "invalid 9p message")
}

impl<'a, 'b: 'a> FcallDecoder<'b> {
    fn decode_u8(&'a mut self) -> std::io::Result<u8> {
        if let Some(v) = self.buf.first() {
            self.buf = &self.buf[1..];
            Ok(*v)
        } else {
            Err(invalid_9p_msg())
        }
    }

    fn decode_u16(&'a mut self) -> std::io::Result<u16> {
        if self.buf.len() >= 2 {
            let v = u16::from_le_bytes(self.buf[0..2].try_into().unwrap());
            self.buf = &self.buf[2..];
            Ok(v)
        } else {
            Err(invalid_9p_msg())
        }
    }

    fn decode_u32(&'a mut self) -> std::io::Result<u32> {
        if self.buf.len() >= 4 {
            let v = u32::from_le_bytes(self.buf[0..4].try_into().unwrap());
            self.buf = &self.buf[4..];
            Ok(v)
        } else {
            Err(invalid_9p_msg())
        }
    }

    fn decode_u64(&'a mut self) -> std::io::Result<u64> {
        if self.buf.len() >= 8 {
            let v = u64::from_le_bytes(self.buf[0..8].try_into().unwrap());
            self.buf = &self.buf[8..];
            Ok(v)
        } else {
            Err(invalid_9p_msg())
        }
    }

    fn decode_str(&mut self) -> std::io::Result<Cow<'b, str>> {
        let n = self.decode_u16()? as usize;
        if self.buf.len() >= n {
            match std::str::from_utf8(&self.buf[..n]) {
                Ok(s) => {
                    self.buf = &self.buf[n..];
                    Ok(Cow::from(s))
                }
                Err(_) => Err(invalid_9p_msg()),
            }
        } else {
            Err(invalid_9p_msg())
        }
    }

    fn decode_data_buf(&mut self) -> std::io::Result<Cow<'b, [u8]>> {
        let n = self.decode_u32()? as usize;
        if self.buf.len() >= n {
            let v = &self.buf[..n];
            self.buf = &self.buf[n..];
            Ok(Cow::from(v))
        } else {
            Err(invalid_9p_msg())
        }
    }

    fn decode_vec_str(&mut self) -> std::io::Result<Vec<Cow<'b, str>>> {
        let len = self.decode_u16()?;
        let mut v = Vec::new();
        for _ in 0..len {
            v.push(self.decode_str()?);
        }
        Ok(v)
    }

    fn decode_vec_qid(&mut self) -> std::io::Result<Vec<Qid>> {
        let len = self.decode_u16()?;
        let mut v = Vec::new();
        for _ in 0..len {
            v.push(self.decode_qid()?);
        }
        Ok(v)
    }

    fn decode_direntrydata(&mut self) -> std::io::Result<DirEntryData<'b>> {
        let end_len = self.buf.len() - self.decode_u32()? as usize;
        let mut v = Vec::new();
        while self.buf.len() > end_len {
            v.push(self.decode_direntry()?);
        }
        Ok(DirEntryData::with(v))
    }

    fn decode_qidtype(&'a mut self) -> std::io::Result<QidType> {
        Ok(QidType::from_bits_truncate(self.decode_u8()?))
    }

    fn decode_locktype(&'a mut self) -> std::io::Result<LockType> {
        Ok(LockType::from_bits_truncate(self.decode_u8()?))
    }

    fn decode_lockstatus(&'a mut self) -> std::io::Result<LockStatus> {
        Ok(LockStatus::from_bits_truncate(self.decode_u8()?))
    }

    fn decode_lockflag(&'a mut self) -> std::io::Result<LockFlag> {
        Ok(LockFlag::from_bits_truncate(self.decode_u32()?))
    }

    fn decode_getattrmask(&'a mut self) -> std::io::Result<GetattrMask> {
        Ok(GetattrMask::from_bits_truncate(self.decode_u64()?))
    }

    fn decode_setattrmask(&'a mut self) -> std::io::Result<SetattrMask> {
        Ok(SetattrMask::from_bits_truncate(self.decode_u32()?))
    }

    fn decode_qid(&mut self) -> std::io::Result<Qid> {
        Ok(Qid {
            typ: self.decode_qidtype()?,
            version: self.decode_u32()?,
            path: self.decode_u64()?,
        })
    }
    fn decode_statfs(&mut self) -> std::io::Result<Statfs> {
        Ok(Statfs {
            typ: self.decode_u32()?,
            bsize: self.decode_u32()?,
            blocks: self.decode_u64()?,
            bfree: self.decode_u64()?,
            bavail: self.decode_u64()?,
            files: self.decode_u64()?,
            ffree: self.decode_u64()?,
            fsid: self.decode_u64()?,
            namelen: self.decode_u32()?,
        })
    }

    fn decode_time(&mut self) -> std::io::Result<Time> {
        Ok(Time {
            sec: self.decode_u64()?,
            nsec: self.decode_u64()?,
        })
    }

    fn decode_stat(&mut self) -> std::io::Result<Stat> {
        Ok(Stat {
            mode: self.decode_u32()?,
            uid: self.decode_u32()?,
            gid: self.decode_u32()?,
            nlink: self.decode_u64()?,
            rdev: self.decode_u64()?,
            size: self.decode_u64()?,
            blksize: self.decode_u64()?,
            blocks: self.decode_u64()?,
            atime: self.decode_time()?,
            mtime: self.decode_time()?,
            ctime: self.decode_time()?,
            btime: self.decode_time()?,
            gen: self.decode_u64()?,
            data_version: self.decode_u64()?,
        })
    }

    fn decode_setattr(&mut self) -> std::io::Result<SetAttr> {
        Ok(SetAttr {
            mode: self.decode_u32()?,
            uid: self.decode_u32()?,
            gid: self.decode_u32()?,
            size: self.decode_u64()?,
            atime: self.decode_time()?,
            mtime: self.decode_time()?,
        })
    }

    fn decode_direntry(&mut self) -> std::io::Result<DirEntry<'b>> {
        Ok(DirEntry {
            qid: self.decode_qid()?,
            offset: self.decode_u64()?,
            typ: self.decode_u8()?,
            name: self.decode_str()?,
        })
    }

    fn decode_flock(&mut self) -> std::io::Result<Flock<'b>> {
        Ok(Flock {
            typ: self.decode_locktype()?,
            flags: self.decode_lockflag()?,
            start: self.decode_u64()?,
            length: self.decode_u64()?,
            proc_id: self.decode_u32()?,
            client_id: self.decode_str()?,
        })
    }

    fn decode_getlock(&mut self) -> std::io::Result<Getlock<'b>> {
        Ok(Getlock {
            typ: self.decode_locktype()?,
            start: self.decode_u64()?,
            length: self.decode_u64()?,
            proc_id: self.decode_u32()?,
            client_id: self.decode_str()?,
        })
    }

    fn decode_rlerror(&mut self) -> std::io::Result<Rlerror> {
        Ok(Rlerror {
            ecode: self.decode_u32()?,
        })
    }

    fn decode_tattach(&mut self) -> std::io::Result<Tattach<'b>> {
        Ok(Tattach {
            fid: self.decode_u32()?,
            afid: self.decode_u32()?,
            uname: self.decode_str()?,
            aname: self.decode_str()?,
            n_uname: self.decode_u32()?,
        })
    }

    fn decode_rattach(&mut self) -> std::io::Result<Rattach> {
        Ok(Rattach {
            qid: self.decode_qid()?,
        })
    }

    fn decode_tstatfs(&mut self) -> std::io::Result<Tstatfs> {
        Ok(Tstatfs {
            fid: self.decode_u32()?,
        })
    }

    fn decode_rstatfs(&mut self) -> std::io::Result<Rstatfs> {
        Ok(Rstatfs {
            statfs: self.decode_statfs()?,
        })
    }

    fn decode_tlopen(&mut self) -> std::io::Result<Tlopen> {
        Ok(Tlopen {
            fid: self.decode_u32()?,
            flags: LOpenFlags::from_bits_truncate(self.decode_u32()?),
        })
    }

    fn decode_rlopen(&mut self) -> std::io::Result<Rlopen> {
        Ok(Rlopen {
            qid: self.decode_qid()?,
            iounit: self.decode_u32()?,
        })
    }

    fn decode_tlcreate(&mut self) -> std::io::Result<Tlcreate<'b>> {
        Ok(Tlcreate {
            fid: self.decode_u32()?,
            name: self.decode_str()?,
            flags: LOpenFlags::from_bits_truncate(self.decode_u32()?),
            mode: self.decode_u32()?,
            gid: self.decode_u32()?,
        })
    }

    fn decode_rlcreate(&mut self) -> std::io::Result<Rlcreate> {
        Ok(Rlcreate {
            qid: self.decode_qid()?,
            iounit: self.decode_u32()?,
        })
    }

    fn decode_tsymlink(&mut self) -> std::io::Result<Tsymlink<'b>> {
        Ok(Tsymlink {
            fid: self.decode_u32()?,
            name: self.decode_str()?,
            symtgt: self.decode_str()?,
            gid: self.decode_u32()?,
        })
    }

    fn decode_rsymlink(&mut self) -> std::io::Result<Rsymlink> {
        Ok(Rsymlink {
            qid: self.decode_qid()?,
        })
    }

    fn decode_tmknod(&mut self) -> std::io::Result<Tmknod<'b>> {
        Ok(Tmknod {
            dfid: self.decode_u32()?,
            name: self.decode_str()?,
            mode: self.decode_u32()?,
            major: self.decode_u32()?,
            minor: self.decode_u32()?,
            gid: self.decode_u32()?,
        })
    }

    fn decode_rmknod(&mut self) -> std::io::Result<Rmknod> {
        Ok(Rmknod {
            qid: self.decode_qid()?,
        })
    }

    fn decode_trename(&mut self) -> std::io::Result<Trename<'b>> {
        Ok(Trename {
            fid: self.decode_u32()?,
            dfid: self.decode_u32()?,
            name: self.decode_str()?,
        })
    }

    fn decode_rrename(&mut self) -> std::io::Result<Rrename> {
        Ok(Rrename {})
    }

    fn decode_treadlink(&mut self) -> std::io::Result<Treadlink> {
        Ok(Treadlink {
            fid: self.decode_u32()?,
        })
    }

    fn decode_rreadlink(&mut self) -> std::io::Result<Rreadlink<'b>> {
        Ok(Rreadlink {
            target: self.decode_str()?,
        })
    }

    fn decode_tgetattr(&mut self) -> std::io::Result<Tgetattr> {
        Ok(Tgetattr {
            fid: self.decode_u32()?,
            req_mask: self.decode_getattrmask()?,
        })
    }

    fn decode_rgetattr(&mut self) -> std::io::Result<Rgetattr> {
        Ok(Rgetattr {
            valid: self.decode_getattrmask()?,
            qid: self.decode_qid()?,
            stat: self.decode_stat()?,
        })
    }

    fn decode_tsetattr(&mut self) -> std::io::Result<Tsetattr> {
        Ok(Tsetattr {
            fid: self.decode_u32()?,
            valid: self.decode_setattrmask()?,
            stat: self.decode_setattr()?,
        })
    }

    fn decode_rsetattr(&mut self) -> std::io::Result<Rsetattr> {
        Ok(Rsetattr {})
    }

    fn decode_txattrwalk(&mut self) -> std::io::Result<Txattrwalk<'b>> {
        Ok(Txattrwalk {
            fid: self.decode_u32()?,
            new_fid: self.decode_u32()?,
            name: self.decode_str()?,
        })
    }

    fn decode_rxattrwalk(&mut self) -> std::io::Result<Rxattrwalk> {
        Ok(Rxattrwalk {
            size: self.decode_u64()?,
        })
    }

    fn decode_txattrcreate(&mut self) -> std::io::Result<Txattrcreate<'b>> {
        Ok(Txattrcreate {
            fid: self.decode_u32()?,
            name: self.decode_str()?,
            attr_size: self.decode_u64()?,
            flags: self.decode_u32()?,
        })
    }

    fn decode_rxattrcreate(&mut self) -> std::io::Result<Rxattrcreate> {
        Ok(Rxattrcreate {})
    }

    fn decode_treaddir(&mut self) -> std::io::Result<Treaddir> {
        Ok(Treaddir {
            fid: self.decode_u32()?,
            offset: self.decode_u64()?,
            count: self.decode_u32()?,
        })
    }

    fn decode_rreaddir(&mut self) -> std::io::Result<Rreaddir<'b>> {
        Ok(Rreaddir {
            data: self.decode_direntrydata()?,
        })
    }

    fn decode_tfsync(&mut self) -> std::io::Result<Tfsync> {
        Ok(Tfsync {
            fid: self.decode_u32()?,
        })
    }

    fn decode_rfsync(&mut self) -> std::io::Result<Rfsync> {
        Ok(Rfsync {})
    }

    fn decode_tlock(&mut self) -> std::io::Result<Tlock<'b>> {
        Ok(Tlock {
            fid: self.decode_u32()?,
            flock: self.decode_flock()?,
        })
    }

    fn decode_rlock(&mut self) -> std::io::Result<Rlock> {
        Ok(Rlock {
            status: self.decode_lockstatus()?,
        })
    }

    fn decode_tgetlock(&mut self) -> std::io::Result<Tgetlock<'b>> {
        Ok(Tgetlock {
            fid: self.decode_u32()?,
            flock: self.decode_getlock()?,
        })
    }

    fn decode_rgetlock(&mut self) -> std::io::Result<Rgetlock<'b>> {
        Ok(Rgetlock {
            flock: self.decode_getlock()?,
        })
    }

    fn decode_tlink(&mut self) -> std::io::Result<Tlink<'b>> {
        Ok(Tlink {
            dfid: self.decode_u32()?,
            fid: self.decode_u32()?,
            name: self.decode_str()?,
        })
    }

    fn decode_rlink(&mut self) -> std::io::Result<Rlink> {
        Ok(Rlink {})
    }

    fn decode_tmkdir(&mut self) -> std::io::Result<Tmkdir<'b>> {
        Ok(Tmkdir {
            dfid: self.decode_u32()?,
            name: self.decode_str()?,
            mode: self.decode_u32()?,
            gid: self.decode_u32()?,
        })
    }

    fn decode_rmkdir(&mut self) -> std::io::Result<Rmkdir> {
        Ok(Rmkdir {
            qid: self.decode_qid()?,
        })
    }

    fn decode_trenameat(&mut self) -> std::io::Result<Trenameat<'b>> {
        Ok(Trenameat {
            olddfid: self.decode_u32()?,
            oldname: self.decode_str()?,
            newdfid: self.decode_u32()?,
            newname: self.decode_str()?,
        })
    }

    fn decode_rrenameat(&mut self) -> std::io::Result<Rrenameat> {
        Ok(Rrenameat {})
    }

    fn decode_tunlinkat(&mut self) -> std::io::Result<Tunlinkat<'b>> {
        Ok(Tunlinkat {
            dfid: self.decode_u32()?,
            name: self.decode_str()?,
            flags: self.decode_u32()?,
        })
    }

    fn decode_runlinkat(&mut self) -> std::io::Result<Runlinkat> {
        Ok(Runlinkat {})
    }

    fn decode_tauth(&mut self) -> std::io::Result<Tauth<'b>> {
        Ok(Tauth {
            afid: self.decode_u32()?,
            uname: self.decode_str()?,
            aname: self.decode_str()?,
            n_uname: self.decode_u32()?,
        })
    }

    fn decode_rauth(&mut self) -> std::io::Result<Rauth> {
        Ok(Rauth {
            aqid: self.decode_qid()?,
        })
    }

    fn decode_tversion(&mut self) -> std::io::Result<Tversion<'b>> {
        Ok(Tversion {
            msize: self.decode_u32()?,
            version: self.decode_str()?,
        })
    }

    fn decode_rversion(&mut self) -> std::io::Result<Rversion<'b>> {
        Ok(Rversion {
            msize: self.decode_u32()?,
            version: self.decode_str()?,
        })
    }

    fn decode_tflush(&mut self) -> std::io::Result<Tflush> {
        Ok(Tflush {
            oldtag: self.decode_u16()?,
        })
    }

    fn decode_rflush(&mut self) -> std::io::Result<Rflush> {
        Ok(Rflush {})
    }

    fn decode_twalk(&mut self) -> std::io::Result<Twalk<'b>> {
        Ok(Twalk {
            fid: self.decode_u32()?,
            new_fid: self.decode_u32()?,
            wnames: self.decode_vec_str()?,
        })
    }

    fn decode_rwalk(&mut self) -> std::io::Result<Rwalk> {
        Ok(Rwalk {
            wqids: self.decode_vec_qid()?,
        })
    }

    fn decode_tread(&mut self) -> std::io::Result<Tread> {
        Ok(Tread {
            fid: self.decode_u32()?,
            offset: self.decode_u64()?,
            count: self.decode_u32()?,
        })
    }

    fn decode_rread(&mut self) -> std::io::Result<Rread<'b>> {
        Ok(Rread {
            data: self.decode_data_buf()?,
        })
    }

    fn decode_twrite(&mut self) -> std::io::Result<Twrite<'b>> {
        Ok(Twrite {
            fid: self.decode_u32()?,
            offset: self.decode_u64()?,
            data: self.decode_data_buf()?,
        })
    }

    fn decode_rwrite(&mut self) -> std::io::Result<Rwrite> {
        Ok(Rwrite {
            count: self.decode_u32()?,
        })
    }

    fn decode_tclunk(&mut self) -> std::io::Result<Tclunk> {
        Ok(Tclunk {
            fid: self.decode_u32()?,
        })
    }

    fn decode_rclunk(&mut self) -> std::io::Result<Rclunk> {
        Ok(Rclunk {})
    }

    fn decode_tremove(&mut self) -> std::io::Result<Tremove> {
        Ok(Tremove {
            fid: self.decode_u32()?,
        })
    }

    fn decode_rremove(&mut self) -> std::io::Result<Rremove> {
        Ok(Rremove {})
    }

    fn decode(&mut self) -> std::io::Result<TaggedFcall<'b>> {
        let msg_type = FcallType::from_u8(self.decode_u8()?);
        let tag = self.decode_u16()?;
        let fcall = match msg_type {
            Some(FcallType::Rlerror) => Fcall::Rlerror(self.decode_rlerror()?),
            Some(FcallType::Tattach) => Fcall::Tattach(self.decode_tattach()?),
            Some(FcallType::Rattach) => Fcall::Rattach(self.decode_rattach()?),
            Some(FcallType::Tstatfs) => Fcall::Tstatfs(self.decode_tstatfs()?),
            Some(FcallType::Rstatfs) => Fcall::Rstatfs(self.decode_rstatfs()?),
            Some(FcallType::Tlopen) => Fcall::Tlopen(self.decode_tlopen()?),
            Some(FcallType::Rlopen) => Fcall::Rlopen(self.decode_rlopen()?),
            Some(FcallType::Tlcreate) => Fcall::Tlcreate(self.decode_tlcreate()?),
            Some(FcallType::Rlcreate) => Fcall::Rlcreate(self.decode_rlcreate()?),
            Some(FcallType::Tsymlink) => Fcall::Tsymlink(self.decode_tsymlink()?),
            Some(FcallType::Rsymlink) => Fcall::Rsymlink(self.decode_rsymlink()?),
            Some(FcallType::Tmknod) => Fcall::Tmknod(self.decode_tmknod()?),
            Some(FcallType::Rmknod) => Fcall::Rmknod(self.decode_rmknod()?),
            Some(FcallType::Trename) => Fcall::Trename(self.decode_trename()?),
            Some(FcallType::Rrename) => Fcall::Rrename(self.decode_rrename()?),
            Some(FcallType::Treadlink) => Fcall::Treadlink(self.decode_treadlink()?),
            Some(FcallType::Rreadlink) => Fcall::Rreadlink(self.decode_rreadlink()?),
            Some(FcallType::Tgetattr) => Fcall::Tgetattr(self.decode_tgetattr()?),
            Some(FcallType::Rgetattr) => Fcall::Rgetattr(self.decode_rgetattr()?),
            Some(FcallType::Tsetattr) => Fcall::Tsetattr(self.decode_tsetattr()?),
            Some(FcallType::Rsetattr) => Fcall::Rsetattr(self.decode_rsetattr()?),
            Some(FcallType::Txattrwalk) => Fcall::Txattrwalk(self.decode_txattrwalk()?),
            Some(FcallType::Rxattrwalk) => Fcall::Rxattrwalk(self.decode_rxattrwalk()?),
            Some(FcallType::Txattrcreate) => Fcall::Txattrcreate(self.decode_txattrcreate()?),
            Some(FcallType::Rxattrcreate) => Fcall::Rxattrcreate(self.decode_rxattrcreate()?),
            Some(FcallType::Treaddir) => Fcall::Treaddir(self.decode_treaddir()?),
            Some(FcallType::Rreaddir) => Fcall::Rreaddir(self.decode_rreaddir()?),
            Some(FcallType::Tfsync) => Fcall::Tfsync(self.decode_tfsync()?),
            Some(FcallType::Rfsync) => Fcall::Rfsync(self.decode_rfsync()?),
            Some(FcallType::Tlock) => Fcall::Tlock(self.decode_tlock()?),
            Some(FcallType::Rlock) => Fcall::Rlock(self.decode_rlock()?),
            Some(FcallType::Tgetlock) => Fcall::Tgetlock(self.decode_tgetlock()?),
            Some(FcallType::Rgetlock) => Fcall::Rgetlock(self.decode_rgetlock()?),
            Some(FcallType::Tlink) => Fcall::Tlink(self.decode_tlink()?),
            Some(FcallType::Rlink) => Fcall::Rlink(self.decode_rlink()?),
            Some(FcallType::Tmkdir) => Fcall::Tmkdir(self.decode_tmkdir()?),
            Some(FcallType::Rmkdir) => Fcall::Rmkdir(self.decode_rmkdir()?),
            Some(FcallType::Trenameat) => Fcall::Trenameat(self.decode_trenameat()?),
            Some(FcallType::Rrenameat) => Fcall::Rrenameat(self.decode_rrenameat()?),
            Some(FcallType::Tunlinkat) => Fcall::Tunlinkat(self.decode_tunlinkat()?),
            Some(FcallType::Runlinkat) => Fcall::Runlinkat(self.decode_runlinkat()?),
            Some(FcallType::Tauth) => Fcall::Tauth(self.decode_tauth()?),
            Some(FcallType::Rauth) => Fcall::Rauth(self.decode_rauth()?),
            Some(FcallType::Tversion) => Fcall::Tversion(self.decode_tversion()?),
            Some(FcallType::Rversion) => Fcall::Rversion(self.decode_rversion()?),
            Some(FcallType::Tflush) => Fcall::Tflush(self.decode_tflush()?),
            Some(FcallType::Rflush) => Fcall::Rflush(self.decode_rflush()?),
            Some(FcallType::Twalk) => Fcall::Twalk(self.decode_twalk()?),
            Some(FcallType::Rwalk) => Fcall::Rwalk(self.decode_rwalk()?),
            Some(FcallType::Tread) => Fcall::Tread(self.decode_tread()?),
            Some(FcallType::Rread) => Fcall::Rread(self.decode_rread()?),
            Some(FcallType::Twrite) => Fcall::Twrite(self.decode_twrite()?),
            Some(FcallType::Rwrite) => Fcall::Rwrite(self.decode_rwrite()?),
            Some(FcallType::Tclunk) => Fcall::Tclunk(self.decode_tclunk()?),
            Some(FcallType::Rclunk) => Fcall::Rclunk(self.decode_rclunk()?),
            Some(FcallType::Tremove) => Fcall::Tremove(self.decode_tremove()?),
            Some(FcallType::Rremove) => Fcall::Rremove(self.decode_rremove()?),
            None => return Err(invalid_9p_msg()),
        };
        Ok(TaggedFcall { tag, fcall })
    }
}
