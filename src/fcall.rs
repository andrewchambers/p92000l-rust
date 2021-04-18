//! 9P protocol data types and constants.
//!
//! # Protocol
//! 9P2000.L

use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use enum_primitive::*;
use num_traits::FromPrimitive;
use std::fs;
use std::io::Read;
use std::ops::{Shl, Shr};
use std::os::unix::fs::MetadataExt;

/// 9P2000 version string
pub const P92000: &str = "9P2000";

/// 9P2000.L version string
pub const P92000L: &str = "9P2000.L";

/*
 * 9P magic numbers
 */
/// Special tag which `Tversion`/`Rversion` must use as `tag`
pub const NOTAG: u16 = !0;

/// Special value which `Tattach` with no auth must use as `afid`
///
/// If the client does not wish to authenticate the connection, or knows that authentication is
/// not required, the afid field in the attach message should be set to `NOFID`
pub const NOFID: u32 = !0;

/// Special uid which `Tauth`/`Tattach` use as `n_uname` to indicate no uid is specified
pub const NONUNAME: u32 = !0;

/// Ample room for `Twrite`/`Rread` header
///
/// size[4] Tread/Twrite[2] tag[2] fid[4] offset[8] count[4]
pub const IOHDRSZ: u32 = 24;

/// Room for readdir header
pub const READDIRHDRSZ: u32 = 24;

/// v9fs default port
pub const V9FS_PORT: u16 = 564;

/// Old 9P2000 protocol types
///
/// Types in this module are not used 9P2000.L
pub mod p92000 {
    /// The type of I/O
    ///
    /// Open mode to be checked against the permissions for the file.
    pub mod om {
        /// Open for read
        pub const READ: u8 = 0;
        /// Write
        pub const WRITE: u8 = 1;
        /// Read and write
        pub const RDWR: u8 = 2;
        /// Execute, == read but check execute permission
        pub const EXEC: u8 = 3;
        /// Or'ed in (except for exec), truncate file first
        pub const TRUNC: u8 = 16;
        /// Or'ed in, close on exec
        pub const CEXEC: u8 = 32;
        /// Or'ed in, remove on close
        pub const RCLOSE: u8 = 64;
    }

    /// Bits in Stat.mode
    pub mod dm {
        /// Mode bit for directories
        pub const DIR: u32 = 0x80000000;
        /// Mode bit for append only files
        pub const APPEND: u32 = 0x40000000;
        /// Mode bit for exclusive use files
        pub const EXCL: u32 = 0x20000000;
        /// Mode bit for mounted channel
        pub const MOUNT: u32 = 0x10000000;
        /// Mode bit for authentication file
        pub const AUTH: u32 = 0x08000000;
        /// Mode bit for non-backed-up files
        pub const TMP: u32 = 0x04000000;
        /// Mode bit for read permission
        pub const READ: u32 = 0x4;
        /// Mode bit for write permission
        pub const WRITE: u32 = 0x2;
        /// Mode bit for execute permission
        pub const EXEC: u32 = 0x1;
    }

    /// Plan 9 Namespace metadata (somewhat like a unix fstat)
    ///
    /// NOTE: Defined as `Dir` in libc.h of Plan 9
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Stat {
        /// Server type
        pub typ: u16,
        /// Server subtype
        pub dev: u32,
        /// Unique id from server
        pub qid: super::Qid,
        /// Permissions
        pub mode: u32,
        /// Last read time
        pub atime: u32,
        /// Last write time
        pub mtime: u32,
        /// File length
        pub length: u64,
        /// Last element of path
        pub name: String,
        /// Owner name
        pub uid: String,
        /// Group name
        pub gid: String,
        /// Last modifier name
        pub muid: String,
    }

    impl Stat {
        /// Get the current size of the stat
        pub fn size(&self) -> u16 {
            (2 + 4
                + 13
                + 4
                + 4
                + 4
                + 8
                + (2 * 4)
                + self.name.len()
                + self.uid.len()
                + self.gid.len()
                + self.muid.len()) as u16
        }
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

/// Server side data type for path tracking
///
/// The server's unique identification for the file being accessed
///
/// # Protocol
/// 9P2000/9P2000.L
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Qid {
    /// Specify whether the file is a directory, append-only file, etc.
    pub typ: QidType,
    /// Version number for a file; typically, it is incremented every time the file is modified
    pub version: u32,
    /// An integer which is unique among all files in the hierarchy
    pub path: u64,
}

/// Filesystem information corresponding to `struct statfs` of Linux.
///
/// # Protocol
/// 9P2000.L
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Statfs {
    /// Type of file system
    pub typ: u32,
    /// Optimal transfer block size
    pub bsize: u32,
    /// Total data blocks in file system
    pub blocks: u64,
    /// Free blocks in fs
    pub bfree: u64,
    /// Free blocks avail to non-superuser
    pub bavail: u64,
    /// Total file nodes in file system
    pub files: u64,
    /// Free file nodes in fs
    pub ffree: u64,
    /// Filesystem ID
    pub fsid: u64,
    /// Maximum length of filenames
    pub namelen: u32,
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

/// Time struct
///
/// # Protocol
/// 9P2000.L
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    pub sec: u64,
    pub nsec: u64,
}

/// File attributes corresponding to `struct stat` of Linux.
///
/// Stat can be constructed from `std::fs::Metadata` via From trait
///
/// # Protocol
/// 9P2000.L
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stat {
    /// Protection
    pub mode: u32,
    /// User ID of owner
    pub uid: u32,
    /// Group ID of owner
    pub gid: u32,
    /// Number of hard links
    pub nlink: u64,
    /// Device ID (if special file)
    pub rdev: u64,
    /// Total size, in bytes
    pub size: u64,
    /// Blocksize for file system I/O
    pub blksize: u64,
    /// Number of 512B blocks allocated
    pub blocks: u64,
    /// Time of last access
    pub atime: Time,
    /// Time of last modification
    pub mtime: Time,
    /// Time of last status change
    pub ctime: Time,
}

impl From<fs::Metadata> for Stat {
    fn from(attr: fs::Metadata) -> Self {
        From::from(&attr)
    }
}

// Default conversion from metadata of libstd
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
        }
    }
}

/// Subset of `Stat` used for `Tsetattr`
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SetAttr {
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime: Time,
    pub mtime: Time,
}

/// Directory entry used in `Rreaddir`
///
/// # Protocol
/// 9P2000.L
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirEntry {
    /// Qid for this directory
    pub qid: Qid,
    /// The index of this entry
    pub offset: u64,
    /// Corresponds to `d_type` of `struct dirent`
    ///
    /// Use `0` if you can't set this properly. It might be enough.
    pub typ: u8,
    /// Directory name
    pub name: String,
}

impl DirEntry {
    pub fn size(&self) -> u32 {
        (13 + 8 + 1 + 2 + self.name.len()) as u32
    }
}

/// Directory entry array
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirEntryData {
    pub data: Vec<DirEntry>,
}

impl DirEntryData {
    pub fn new() -> DirEntryData {
        Self::with(Vec::new())
    }
    pub fn with(v: Vec<DirEntry>) -> DirEntryData {
        DirEntryData { data: v }
    }
    pub fn data(&self) -> &[DirEntry] {
        &self.data
    }
    pub fn size(&self) -> u32 {
        self.data.iter().fold(0, |a, e| a + e.size()) as u32
    }
    pub fn push(&mut self, entry: DirEntry) {
        self.data.push(entry);
    }
}

/// Data type used in `Rread` and `Twrite`
///
/// # Protocol
/// 9P2000/9P2000.L
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Data(pub Vec<u8>);

/// Similar to Linux `struct flock`
///
/// # Protocol
/// 9P2000.L
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flock {
    pub typ: LockType,
    pub flags: LockFlag,
    pub start: u64,
    pub length: u64,
    pub proc_id: u32,
    pub client_id: String,
}

/// Getlock structure
///
/// # Protocol
/// 9P2000.L
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Getlock {
    pub typ: LockType,
    pub start: u64,
    pub length: u64,
    pub proc_id: u32,
    pub client_id: String,
}

// Commented out the types not used in 9P2000.L
enum_from_primitive! {
    #[doc = "Message type, 9P operations"]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum MsgType {
        // 9P2000.L
        Tlerror         = 6,    // Illegal, never used
        Rlerror,
        Tstatfs         = 8,
        Rstatfs,
        Tlopen          = 12,
        Rlopen,
        Tlcreate        = 14,
        Rlcreate,
        Tsymlink        = 16,
        Rsymlink,
        Tmknod          = 18,
        Rmknod,
        Trename         = 20,
        Rrename,
        Treadlink       = 22,
        Rreadlink,
        Tgetattr        = 24,
        Rgetattr,
        Tsetattr        = 26,
        Rsetattr,
        Txattrwalk      = 30,
        Rxattrwalk,
        Txattrcreate    = 32,
        Rxattrcreate,
        Treaddir        = 40,
        Rreaddir,
        Tfsync          = 50,
        Rfsync,
        Tlock           = 52,
        Rlock,
        Tgetlock        = 54,
        Rgetlock,
        Tlink           = 70,
        Rlink,
        Tmkdir          = 72,
        Rmkdir,
        Trenameat       = 74,
        Rrenameat,
        Tunlinkat       = 76,
        Runlinkat,

        // 9P2000
        Tversion        = 100,
        Rversion,
        Tauth           = 102,
        Rauth,
        Tattach         = 104,
        Rattach,
        //Terror          = 106,  // Illegal, never used
        //Rerror,
        Tflush          = 108,
        Rflush,
        Twalk           = 110,
        Rwalk,
        //Topen           = 112,
        //Ropen,
        //Tcreate         = 114,
        //Rcreate,
        Tread           = 116,
        Rread,
        Twrite          = 118,
        Rwrite,
        Tclunk          = 120,
        Rclunk,
        Tremove         = 122,
        Rremove,
        //Tstat           = 124,
        //Rstat,
        //Twstat          = 126,
        //Rwstat,
    }
}

impl MsgType {
    /// If the message type is T-message
    pub fn is_t(&self) -> bool {
        !self.is_r()
    }

    /// If the message type is R-message
    pub fn is_r(&self) -> bool {
        use MsgType::*;
        match *self {
            Rlerror | Rstatfs | Rlopen | Rlcreate | Rsymlink | Rmknod | Rrename | Rreadlink
            | Rgetattr | Rsetattr | Rxattrwalk | Rxattrcreate | Rreaddir | Rfsync | Rlock
            | Rgetlock | Rlink | Rmkdir | Rrenameat | Runlinkat | Rversion | Rauth | Rattach
            | Rflush | Rwalk | Rread | Rwrite | Rclunk | Rremove => true,
            _ => false,
        }
    }
}

impl<'a> From<&'a Fcall> for MsgType {
    fn from(fcall: &'a Fcall) -> MsgType {
        match *fcall {
            Fcall::Rlerror(_) => MsgType::Rlerror,
            Fcall::Tstatfs(_) => MsgType::Tstatfs,
            Fcall::Rstatfs(_) => MsgType::Rstatfs,
            Fcall::Tlopen(_) => MsgType::Tlopen,
            Fcall::Rlopen(_) => MsgType::Rlopen,
            Fcall::Tlcreate(_) => MsgType::Tlcreate,
            Fcall::Rlcreate(_) => MsgType::Rlcreate,
            Fcall::Tsymlink(_) => MsgType::Tsymlink,
            Fcall::Rsymlink(_) => MsgType::Rsymlink,
            Fcall::Tmknod(_) => MsgType::Tmknod,
            Fcall::Rmknod(_) => MsgType::Rmknod,
            Fcall::Trename(_) => MsgType::Trename,
            Fcall::Rrename(_) => MsgType::Rrename,
            Fcall::Treadlink(_) => MsgType::Treadlink,
            Fcall::Rreadlink(_) => MsgType::Rreadlink,
            Fcall::Tgetattr(_) => MsgType::Tgetattr,
            Fcall::Rgetattr(_) => MsgType::Rgetattr,
            Fcall::Tsetattr(_) => MsgType::Tsetattr,
            Fcall::Rsetattr(_) => MsgType::Rsetattr,
            Fcall::Txattrwalk(_) => MsgType::Txattrwalk,
            Fcall::Rxattrwalk(_) => MsgType::Rxattrwalk,
            Fcall::Txattrcreate(_) => MsgType::Txattrcreate,
            Fcall::Rxattrcreate(_) => MsgType::Rxattrcreate,
            Fcall::Treaddir(_) => MsgType::Treaddir,
            Fcall::Rreaddir(_) => MsgType::Rreaddir,
            Fcall::Tfsync(_) => MsgType::Tfsync,
            Fcall::Rfsync(_) => MsgType::Rfsync,
            Fcall::Tlock(_) => MsgType::Tlock,
            Fcall::Rlock(_) => MsgType::Rlock,
            Fcall::Tgetlock(_) => MsgType::Tgetlock,
            Fcall::Rgetlock(_) => MsgType::Rgetlock,
            Fcall::Tlink(_) => MsgType::Tlink,
            Fcall::Rlink(_) => MsgType::Rlink,
            Fcall::Tmkdir(_) => MsgType::Tmkdir,
            Fcall::Rmkdir(_) => MsgType::Rmkdir,
            Fcall::Trenameat(_) => MsgType::Trenameat,
            Fcall::Rrenameat(_) => MsgType::Rrenameat,
            Fcall::Tunlinkat(_) => MsgType::Tunlinkat,
            Fcall::Runlinkat(_) => MsgType::Runlinkat,
            Fcall::Tauth(_) => MsgType::Tauth,
            Fcall::Rauth(_) => MsgType::Rauth,
            Fcall::Tattach(_) => MsgType::Tattach,
            Fcall::Rattach(_) => MsgType::Rattach,
            Fcall::Tversion(_) => MsgType::Tversion,
            Fcall::Rversion(_) => MsgType::Rversion,
            Fcall::Tflush(_) => MsgType::Tflush,
            Fcall::Rflush(_) => MsgType::Rflush,
            Fcall::Twalk(_) => MsgType::Twalk,
            Fcall::Rwalk(_) => MsgType::Rwalk,
            Fcall::Tread(_) => MsgType::Tread,
            Fcall::Rread(_) => MsgType::Rread,
            Fcall::Twrite(_) => MsgType::Twrite,
            Fcall::Rwrite(_) => MsgType::Rwrite,
            Fcall::Tclunk(_) => MsgType::Tclunk,
            Fcall::Rclunk(_) => MsgType::Rclunk,
            Fcall::Tremove(_) => MsgType::Tremove,
            Fcall::Rremove(_) => MsgType::Rremove,
        }
    }
}

macro_rules! messages{
    (
    $(
     $struct_name:ident {
        $(
        $field_name:ident : $field_type:ty
        ),*$(,)*
    })+
    ) => {
            $(
            #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $struct_name{
                $(
                pub $field_name : $field_type,
                )*
            }

            impl From<$struct_name> for Fcall {
                fn from(v: $struct_name) -> Fcall {
                    Fcall::$struct_name(v)
                }
            }
            )*
    }
}

messages! {

    Rlerror {
        ecode: u32,
    }

    Tattach {
        fid: u32,
        afid: u32,
        uname: String,
        aname: String,
        n_uname: u32,
    }

    Rattach {
      qid: Qid,
    }

    Tstatfs {
        fid: u32,
    }

    Rstatfs {
        statfs: Statfs,
    }

    Tlopen {
        fid: u32,
        flags: u32,
    }

    Rlopen {
        qid: Qid,
        iounit: u32,
    }

    Tlcreate {
        fid: u32,
        name: String,
        flags: u32,
        mode: u32,
        gid: u32,
    }

    Rlcreate {
        qid: Qid,
        iounit: u32,
    }

    Tsymlink {
        fid: u32,
        name: String,
        symtgt: String,
        gid: u32,
    }

    Rsymlink {
        qid: Qid,
    }

    Tmknod {
        dfid: u32,
        name: String,
        mode: u32,
        major: u32,
        minor: u32,
        gid: u32,
    }

    Rmknod {
        qid: Qid,
    }

    Trename {
        fid: u32,
        dfid: u32,
        name: String,
    }

    Rrename {}

    Treadlink {
        fid: u32,
    }

    Rreadlink {
        target: String,
    }

    Tgetattr {
        fid: u32,
        req_mask: GetattrMask,
    }

    Rgetattr {
        valid: GetattrMask,
        qid: Qid,
        stat: Stat,
    }

    Tsetattr {
        fid: u32,
        valid: SetattrMask,
        stat: SetAttr,
    }

    Rsetattr {}

    Txattrwalk {
        fid: u32,
        newfid: u32,
        name: String,
    }

    Rxattrwalk {
        size: u64,
    }

    Txattrcreate {
        fid: u32,
        name: String,
        attr_size: u64,
        flags: u32,
    }

    Rxattrcreate {}

    Treaddir {
        fid: u32,
        offset: u64,
        count: u32,
    }

    Rreaddir {
        data: DirEntryData,
    }

    Tfsync {
        fid: u32,
    }

    Rfsync {}

    Tlock {
        fid: u32,
        flock: Flock,
    }

    Rlock {
        status: LockStatus,
    }

    Tgetlock {
        fid: u32,
        flock: Getlock,
    }

    Rgetlock {
        flock: Getlock,
    }

    Tlink {
        dfid: u32,
        fid: u32,
        name: String,
    }

    Rlink {}

    Tmkdir {
        dfid: u32,
        name: String,
        mode: u32,
        gid: u32,
    }

    Rmkdir {
        qid: Qid,
    }

    Trenameat {
        olddirfid: u32,
        oldname: String,
        newdirfid: u32,
        newname: String,
    }

    Rrenameat {}

    Tunlinkat {
        dirfd: u32,
        name: String,
        flags: u32,
    }

    Runlinkat {}

    Tauth {
        afid: u32,
        uname: String,
        aname: String,
        n_uname: u32,
    }

    Rauth {
        aqid: Qid,
    }

    Tversion {
        msize: u32,
        version: String,
    }

    Rversion {
        msize: u32,
        version: String,
    }

    Tflush {
        oldtag: u16,
    }

    Rflush {}

    Twalk {
        fid: u32,
        newfid: u32,
        wnames: Vec<String>,
    }

    Rwalk {
        wqids: Vec<Qid>,
    }

    Tread {
        fid: u32,
        offset: u64,
        count: u32,
    }

    Rread {
        data: Data,
    }

    Twrite {
        fid: u32,
        offset: u64,
        data: Data,
    }

    Rwrite {
        count: u32,
    }

    Tclunk {
        fid: u32,
    }

    Rclunk {}

    Tremove {
        fid: u32,
    }

    Rremove {}
}

impl From<std::io::Error> for Rlerror {
    fn from(err: std::io::Error) -> Self {
        use super::errno::*;
        use std::io::ErrorKind::*;

        let ecode = match err.kind() {
            NotFound => ENOENT,
            PermissionDenied => EPERM,
            ConnectionRefused => ECONNREFUSED,
            ConnectionReset => ECONNRESET,
            ConnectionAborted => ECONNABORTED,
            NotConnected => ENOTCONN,
            AddrInUse => EADDRINUSE,
            AddrNotAvailable => EADDRNOTAVAIL,
            BrokenPipe => EPIPE,
            AlreadyExists => EALREADY,
            WouldBlock => EAGAIN,
            InvalidInput => EINVAL,
            InvalidData => EINVAL,
            TimedOut => ETIMEDOUT,
            WriteZero => EAGAIN,
            Interrupted => EINTR,
            Other | _ => EIO,
        };

        Rlerror { ecode }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fcall {
    Rlerror(Rlerror),
    Tstatfs(Tstatfs),
    Rstatfs(Rstatfs),
    Tlopen(Tlopen),
    Rlopen(Rlopen),
    Tlcreate(Tlcreate),
    Rlcreate(Rlcreate),
    Tsymlink(Tsymlink),
    Rsymlink(Rsymlink),
    Tmknod(Tmknod),
    Rmknod(Rmknod),
    Trename(Trename),
    Rrename(Rrename),
    Treadlink(Treadlink),
    Rreadlink(Rreadlink),
    Tgetattr(Tgetattr),
    Rgetattr(Rgetattr),
    Tsetattr(Tsetattr),
    Rsetattr(Rsetattr),
    Txattrwalk(Txattrwalk),
    Rxattrwalk(Rxattrwalk),
    Txattrcreate(Txattrcreate),
    Rxattrcreate(Rxattrcreate),
    Treaddir(Treaddir),
    Rreaddir(Rreaddir),
    Tfsync(Tfsync),
    Rfsync(Rfsync),
    Tlock(Tlock),
    Rlock(Rlock),
    Tgetlock(Tgetlock),
    Rgetlock(Rgetlock),
    Tlink(Tlink),
    Rlink(Rlink),
    Tmkdir(Tmkdir),
    Rmkdir(Rmkdir),
    Trenameat(Trenameat),
    Rrenameat(Rrenameat),
    Tunlinkat(Tunlinkat),
    Runlinkat(Runlinkat),
    Tauth(Tauth),
    Rauth(Rauth),
    Tattach(Tattach),
    Rattach(Rattach),
    Tversion(Tversion),
    Rversion(Rversion),
    Tflush(Tflush),
    Rflush(Rflush),
    Twalk(Twalk),
    Rwalk(Rwalk),
    Tread(Tread),
    Rread(Rread),
    Twrite(Twrite),
    Rwrite(Rwrite),
    Tclunk(Tclunk),
    Rclunk(Rclunk),
    Tremove(Tremove),
    Rremove(Rremove),
    // 9P2000 operations not used for 9P2000.L
    //Tauth,
    //Rauth,
    //Rerror,
    //Tattach,
    //Rattach,
    //Topen,
    //Ropen,
    //Tcreate,
    //Rcreate,
    //Tstat,
    //Rstat,
    //Twstat,
    //Rwstat,
}

impl<A: Into<Fcall>, B: Into<Fcall>> From<Result<A, B>> for Fcall {
    fn from(r: Result<A, B>) -> Fcall {
        match r {
            Ok(a) => a.into(),
            Err(b) => b.into(),
        }
    }
}

/// Envelope for 9P messages
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Msg {
    /// Chosen and used by the client to identify the message.
    /// The reply to the message will have the same tag
    pub tag: u16,
    /// Message body encapsulating the various 9P messages
    pub body: Fcall,
}

macro_rules! io_err {
    ($kind:ident, $msg:expr) => {
        ::std::io::Error::new(::std::io::ErrorKind::$kind, $msg)
    };
}

macro_rules! decode {
    ($decoder:expr) => {
        Decodable::decode(&mut $decoder)?
    };

    ($typ:ident, $buf:expr) => {
        $typ::from_bits_truncate(decode!($buf))
    };
}

fn create_buffer(size: usize) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(size);
    unsafe {
        buffer.set_len(size);
    }
    buffer
}

fn read_exact<R: Read + ?Sized>(r: &mut R, size: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = create_buffer(size);
    r.read_exact(&mut buf[..]).and(Ok(buf))
}

/// A serializing specific result to overload operators on `Result`
///
/// # Overloaded operators
/// <<, >>, ?
pub struct SResult<T>(::std::io::Result<T>);

/// A wrapper class of WriteBytesExt to provide operator overloads
/// for serializing
///
/// Operator '<<' serializes the right hand side argument into
/// the left hand side encoder
#[derive(Clone, Debug)]
pub struct Encoder<W> {
    writer: W,
    bytes: usize,
}

impl<W: WriteBytesExt> Encoder<W> {
    pub fn new(writer: W) -> Encoder<W> {
        Encoder { writer, bytes: 0 }
    }

    /// Return total bytes written
    pub fn bytes_written(&self) -> usize {
        self.bytes
    }

    /// Encode data, equivalent to: decoder << data
    pub fn encode<T: Encodable>(&mut self, data: &T) -> std::io::Result<usize> {
        let bytes = data.encode(&mut self.writer)?;
        self.bytes += bytes;
        Ok(bytes)
    }

    /// Get inner writer
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, T: Encodable, W: WriteBytesExt> Shl<&'a T> for Encoder<W> {
    type Output = SResult<Encoder<W>>;
    fn shl(mut self, rhs: &'a T) -> Self::Output {
        match self.encode(rhs) {
            Ok(_) => SResult(Ok(self)),
            Err(e) => SResult(Err(e)),
        }
    }
}

impl<'a, T: Encodable, W: WriteBytesExt> Shl<&'a T> for SResult<Encoder<W>> {
    type Output = Self;
    fn shl(self, rhs: &'a T) -> Self::Output {
        match self.0 {
            Ok(mut encoder) => match encoder.encode(rhs) {
                Ok(_) => SResult(Ok(encoder)),
                Err(e) => SResult(Err(e)),
            },
            Err(e) => SResult(Err(e)),
        }
    }
}

/// A wrapper class of ReadBytesExt to provide operator overloads
/// for deserializing
#[derive(Clone, Debug)]
pub struct Decoder<R> {
    reader: R,
}

impl<R: ReadBytesExt> Decoder<R> {
    pub fn new(reader: R) -> Decoder<R> {
        Decoder { reader }
    }
    pub fn decode<T: Decodable>(&mut self) -> std::io::Result<T> {
        Decodable::decode(&mut self.reader)
    }
    /// Get inner reader
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<'a, T: Decodable, R: ReadBytesExt> Shr<&'a mut T> for Decoder<R> {
    type Output = SResult<Decoder<R>>;
    fn shr(mut self, rhs: &'a mut T) -> Self::Output {
        match self.decode() {
            Ok(r) => {
                *rhs = r;
                SResult(Ok(self))
            }
            Err(e) => SResult(Err(e)),
        }
    }
}

impl<'a, T: Decodable, R: ReadBytesExt> Shr<&'a mut T> for SResult<Decoder<R>> {
    type Output = Self;
    fn shr(self, rhs: &'a mut T) -> Self::Output {
        match self.0 {
            Ok(mut decoder) => match decoder.decode() {
                Ok(r) => {
                    *rhs = r;
                    SResult(Ok(decoder))
                }
                Err(e) => SResult(Err(e)),
            },
            Err(e) => SResult(Err(e)),
        }
    }
}

/// Trait representing a type which can be serialized into binary
pub trait Encodable {
    /// Encode self to w and returns the number of bytes encoded
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize>;
}

impl Encodable for u8 {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        w.write_u8(*self).and(Ok(1))
    }
}

impl Encodable for u16 {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        w.write_u16::<LittleEndian>(*self)
            .and(Ok(2))
    }
}

impl Encodable for u32 {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        w.write_u32::<LittleEndian>(*self)
            .and(Ok(4))
    }
}

impl Encodable for u64 {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        w.write_u64::<LittleEndian>(*self)
            .and(Ok(8))
    }
}

impl Encodable for String {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        let mut bytes = (self.len() as u16).encode(w)?;
        bytes += w.write_all(self.as_bytes()).and(Ok(self.len()))?;
        Ok(bytes)
    }
}

impl Encodable for Qid {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match Encoder::new(w) << &self.typ.bits() << &self.version << &self.path {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for Statfs {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match Encoder::new(w)
            << &self.typ
            << &self.bsize
            << &self.blocks
            << &self.bfree
            << &self.bavail
            << &self.files
            << &self.ffree
            << &self.fsid
            << &self.namelen
        {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for Time {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match Encoder::new(w) << &self.sec << &self.nsec {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for Stat {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match Encoder::new(w)
            << &self.mode
            << &self.uid
            << &self.gid
            << &self.nlink
            << &self.rdev
            << &self.size
            << &self.blksize
            << &self.blocks
            << &self.atime
            << &self.mtime
            << &self.ctime
        {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for SetAttr {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match Encoder::new(w)
            << &self.mode
            << &self.uid
            << &self.gid
            << &self.size
            << &self.atime
            << &self.mtime
        {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for DirEntry {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match Encoder::new(w) << &self.qid << &self.offset << &self.typ << &self.name {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for DirEntryData {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match self
            .data()
            .iter()
            .fold(Encoder::new(w) << &self.size(), |acc, e| acc << e)
        {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for Data {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        let size = self.0.len();
        let bytes = (size as u32).encode(w)? + size;
        w.write_all(&self.0)?;
        Ok(bytes)
    }
}

impl Encodable for Flock {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match Encoder::new(w)
            << &self.typ.bits()
            << &self.flags.bits()
            << &self.start
            << &self.length
            << &self.proc_id
            << &self.client_id
        {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for Getlock {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match Encoder::new(w)
            << &self.typ.bits()
            << &self.start
            << &self.length
            << &self.proc_id
            << &self.client_id
        {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl<T: Encodable> Encodable for Vec<T> {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        match self
            .iter()
            .fold(Encoder::new(w) << &(self.len() as u16), |acc, s| acc << s)
        {
            SResult(Ok(enc)) => Ok(enc.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

impl Encodable for Msg {
    fn encode<W: WriteBytesExt>(&self, w: &mut W) -> std::io::Result<usize> {
        let typ = MsgType::from(&self.body);
        let buf = Encoder::new(w) << &(typ as u8) << &self.tag;

        let buf = match self.body {
            Fcall::Rlerror(Rlerror { ref ecode }) => buf << ecode,
            Fcall::Tstatfs(Tstatfs { ref fid }) => buf << fid,
            Fcall::Rstatfs(Rstatfs { ref statfs }) => buf << statfs,
            Fcall::Tlopen(Tlopen { ref fid, ref flags }) => buf << fid << flags,
            Fcall::Rlopen(Rlopen {
                ref qid,
                ref iounit,
            }) => buf << qid << iounit,
            Fcall::Tlcreate(Tlcreate {
                ref fid,
                ref name,
                ref flags,
                ref mode,
                ref gid,
            }) => buf << fid << name << flags << mode << gid,
            Fcall::Rlcreate(Rlcreate {
                ref qid,
                ref iounit,
            }) => buf << qid << iounit,
            Fcall::Tsymlink(Tsymlink {
                ref fid,
                ref name,
                ref symtgt,
                ref gid,
            }) => buf << fid << name << symtgt << gid,
            Fcall::Rsymlink(Rsymlink { ref qid }) => buf << qid,
            Fcall::Tmknod(Tmknod {
                ref dfid,
                ref name,
                ref mode,
                ref major,
                ref minor,
                ref gid,
            }) => buf << dfid << name << mode << major << minor << gid,
            Fcall::Rmknod(Rmknod { ref qid }) => buf << qid,
            Fcall::Trename(Trename {
                ref fid,
                ref dfid,
                ref name,
            }) => buf << fid << dfid << name,
            Fcall::Rrename(Rrename {}) => buf,
            Fcall::Treadlink(Treadlink { ref fid }) => buf << fid,
            Fcall::Rreadlink(Rreadlink { ref target }) => buf << target,
            Fcall::Tgetattr(Tgetattr {
                ref fid,
                ref req_mask,
            }) => buf << fid << &req_mask.bits(),
            Fcall::Rgetattr(Rgetattr {
                ref valid,
                ref qid,
                ref stat,
            }) => buf << &valid.bits() << qid << stat << &0u64 << &0u64 << &0u64 << &0u64,
            Fcall::Tsetattr(Tsetattr {
                ref fid,
                ref valid,
                ref stat,
            }) => buf << fid << &valid.bits() << stat,
            Fcall::Rsetattr(Rsetattr {}) => buf,
            Fcall::Txattrwalk(Txattrwalk {
                ref fid,
                ref newfid,
                ref name,
            }) => buf << fid << newfid << name,
            Fcall::Rxattrwalk(Rxattrwalk { ref size }) => buf << size,
            Fcall::Txattrcreate(Txattrcreate {
                ref fid,
                ref name,
                ref attr_size,
                ref flags,
            }) => buf << fid << name << attr_size << flags,
            Fcall::Rxattrcreate(Rxattrcreate {}) => buf,
            Fcall::Treaddir(Treaddir {
                ref fid,
                ref offset,
                ref count,
            }) => buf << fid << offset << count,
            Fcall::Rreaddir(Rreaddir { ref data }) => buf << data,
            Fcall::Tfsync(Tfsync { ref fid }) => buf << fid,
            Fcall::Rfsync(Rfsync {}) => buf,
            Fcall::Tlock(Tlock { ref fid, ref flock }) => buf << fid << flock,
            Fcall::Rlock(Rlock { ref status }) => buf << &status.bits(),
            Fcall::Tgetlock(Tgetlock { ref fid, ref flock }) => buf << fid << flock,
            Fcall::Rgetlock(Rgetlock { ref flock }) => buf << flock,
            Fcall::Tlink(Tlink {
                ref dfid,
                ref fid,
                ref name,
            }) => buf << dfid << fid << name,
            Fcall::Rlink(Rlink {}) => buf,
            Fcall::Tmkdir(Tmkdir {
                ref dfid,
                ref name,
                ref mode,
                ref gid,
            }) => buf << dfid << name << mode << gid,
            Fcall::Rmkdir(Rmkdir { ref qid }) => buf << qid,
            Fcall::Trenameat(Trenameat {
                ref olddirfid,
                ref oldname,
                ref newdirfid,
                ref newname,
            }) => buf << olddirfid << oldname << newdirfid << newname,
            Fcall::Rrenameat(Rrenameat {}) => buf,
            Fcall::Tunlinkat(Tunlinkat {
                ref dirfd,
                ref name,
                ref flags,
            }) => buf << dirfd << name << flags,
            Fcall::Runlinkat(Runlinkat {}) => buf,
            Fcall::Tauth(Tauth {
                ref afid,
                ref uname,
                ref aname,
                ref n_uname,
            }) => buf << afid << uname << aname << n_uname,
            Fcall::Rauth(Rauth { ref aqid }) => buf << aqid,
            Fcall::Tattach(Tattach {
                ref fid,
                ref afid,
                ref uname,
                ref aname,
                ref n_uname,
            }) => buf << fid << afid << uname << aname << n_uname,
            Fcall::Rattach(Rattach { ref qid }) => buf << qid,
            Fcall::Tversion(Tversion {
                ref msize,
                ref version,
            }) => buf << msize << version,
            Fcall::Rversion(Rversion {
                ref msize,
                ref version,
            }) => buf << msize << version,
            Fcall::Tflush(Tflush { ref oldtag }) => buf << oldtag,
            Fcall::Rflush(Rflush {}) => buf,
            Fcall::Twalk(Twalk {
                ref fid,
                ref newfid,
                ref wnames,
            }) => buf << fid << newfid << wnames,
            Fcall::Rwalk(Rwalk { ref wqids }) => buf << wqids,
            Fcall::Tread(Tread {
                ref fid,
                ref offset,
                ref count,
            }) => buf << fid << offset << count,
            Fcall::Rread(Rread { ref data }) => buf << data,
            Fcall::Twrite(Twrite {
                ref fid,
                ref offset,
                ref data,
            }) => buf << fid << offset << data,
            Fcall::Rwrite(Rwrite { ref count }) => buf << count,
            Fcall::Tclunk(Tclunk { ref fid }) => buf << fid,
            Fcall::Rclunk(Rclunk {}) => buf,
            Fcall::Tremove(Tremove { ref fid }) => buf << fid,
            Fcall::Rremove(Rremove {}) => buf,
        };

        match buf {
            SResult(Ok(b)) => Ok(b.bytes_written()),
            SResult(Err(e)) => Err(e),
        }
    }
}

pub trait Decodable: Sized {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self>;
}

impl Decodable for u8 {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        r.read_u8()
    }
}

impl Decodable for u16 {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        r.read_u16::<LittleEndian>()
    }
}

impl Decodable for u32 {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        r.read_u32::<LittleEndian>()
    }
}

impl Decodable for u64 {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        r.read_u64::<LittleEndian>()
    }
}

impl Decodable for String {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        let len: u16 = Decodable::decode(r)?;
        String::from_utf8(read_exact(r, len as usize)?)
            .or(Err(io_err!(Other, "Invalid UTF-8 sequence")))
    }
}

impl Decodable for Qid {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        Ok(Qid {
            typ: decode!(QidType, *r),
            version: Decodable::decode(r)?,
            path: Decodable::decode(r)?,
        })
    }
}

impl Decodable for Statfs {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        Ok(Statfs {
            typ: Decodable::decode(r)?,
            bsize: Decodable::decode(r)?,
            blocks: Decodable::decode(r)?,
            bfree: Decodable::decode(r)?,
            bavail: Decodable::decode(r)?,
            files: Decodable::decode(r)?,
            ffree: Decodable::decode(r)?,
            fsid: Decodable::decode(r)?,
            namelen: Decodable::decode(r)?,
        })
    }
}

impl Decodable for Time {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        Ok(Time {
            sec: Decodable::decode(r)?,
            nsec: Decodable::decode(r)?,
        })
    }
}

impl Decodable for Stat {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        Ok(Stat {
            mode: Decodable::decode(r)?,
            uid: Decodable::decode(r)?,
            gid: Decodable::decode(r)?,
            nlink: Decodable::decode(r)?,
            rdev: Decodable::decode(r)?,
            size: Decodable::decode(r)?,
            blksize: Decodable::decode(r)?,
            blocks: Decodable::decode(r)?,
            atime: Decodable::decode(r)?,
            mtime: Decodable::decode(r)?,
            ctime: Decodable::decode(r)?,
        })
    }
}

impl Decodable for SetAttr {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        Ok(SetAttr {
            mode: Decodable::decode(r)?,
            uid: Decodable::decode(r)?,
            gid: Decodable::decode(r)?,
            size: Decodable::decode(r)?,
            atime: Decodable::decode(r)?,
            mtime: Decodable::decode(r)?,
        })
    }
}

impl Decodable for DirEntry {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        Ok(DirEntry {
            qid: Decodable::decode(r)?,
            offset: Decodable::decode(r)?,
            typ: Decodable::decode(r)?,
            name: Decodable::decode(r)?,
        })
    }
}

impl Decodable for DirEntryData {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        let count: u32 = Decodable::decode(r)?;
        let mut data: Vec<DirEntry> = Vec::with_capacity(count as usize);
        for _ in 0..count {
            data.push(Decodable::decode(r)?);
        }
        Ok(DirEntryData::with(data))
    }
}

impl Decodable for Data {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        let len: u32 = Decodable::decode(r)?;
        Ok(Data(read_exact(r, len as usize)?))
    }
}

impl Decodable for Flock {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        Ok(Flock {
            typ: decode!(LockType, *r),
            flags: decode!(LockFlag, *r),
            start: Decodable::decode(r)?,
            length: Decodable::decode(r)?,
            proc_id: Decodable::decode(r)?,
            client_id: Decodable::decode(r)?,
        })
    }
}

impl Decodable for Getlock {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        Ok(Getlock {
            typ: decode!(LockType, *r),
            start: Decodable::decode(r)?,
            length: Decodable::decode(r)?,
            proc_id: Decodable::decode(r)?,
            client_id: Decodable::decode(r)?,
        })
    }
}

impl<T: Decodable> Decodable for Vec<T> {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        let len: u16 = Decodable::decode(r)?;
        let mut buf = Vec::new();
        for _ in 0..len {
            buf.push(Decodable::decode(r)?);
        }
        Ok(buf)
    }
}

impl Decodable for Msg {
    fn decode<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Self> {
        let mut buf = r;

        let msg_type = MsgType::from_u8(decode!(buf));
        let tag = decode!(buf);
        let body = match msg_type {
            Some(MsgType::Rlerror) => Fcall::Rlerror(Rlerror {
                ecode: decode!(buf),
            }),
            Some(MsgType::Tstatfs) => Fcall::Tstatfs(Tstatfs { fid: decode!(buf) }),
            Some(MsgType::Rstatfs) => Fcall::Rstatfs(Rstatfs {
                statfs: decode!(buf),
            }),
            Some(MsgType::Tlopen) => Fcall::Tlopen(Tlopen {
                fid: decode!(buf),
                flags: decode!(buf),
            }),
            Some(MsgType::Rlopen) => Fcall::Rlopen(Rlopen {
                qid: decode!(buf),
                iounit: decode!(buf),
            }),
            Some(MsgType::Tlcreate) => Fcall::Tlcreate(Tlcreate {
                fid: decode!(buf),
                name: decode!(buf),
                flags: decode!(buf),
                mode: decode!(buf),
                gid: decode!(buf),
            }),
            Some(MsgType::Rlcreate) => Fcall::Rlcreate(Rlcreate {
                qid: decode!(buf),
                iounit: decode!(buf),
            }),
            Some(MsgType::Tsymlink) => Fcall::Tsymlink(Tsymlink {
                fid: decode!(buf),
                name: decode!(buf),
                symtgt: decode!(buf),
                gid: decode!(buf),
            }),
            Some(MsgType::Rsymlink) => Fcall::Rsymlink(Rsymlink { qid: decode!(buf) }),
            Some(MsgType::Tmknod) => Fcall::Tmknod(Tmknod {
                dfid: decode!(buf),
                name: decode!(buf),
                mode: decode!(buf),
                major: decode!(buf),
                minor: decode!(buf),
                gid: decode!(buf),
            }),
            Some(MsgType::Rmknod) => Fcall::Rmknod(Rmknod { qid: decode!(buf) }),
            Some(MsgType::Trename) => Fcall::Trename(Trename {
                fid: decode!(buf),
                dfid: decode!(buf),
                name: decode!(buf),
            }),
            Some(MsgType::Rrename) => Fcall::Rrename(Rrename {}),
            Some(MsgType::Treadlink) => Fcall::Treadlink(Treadlink { fid: decode!(buf) }),
            Some(MsgType::Rreadlink) => Fcall::Rreadlink(Rreadlink {
                target: decode!(buf),
            }),
            Some(MsgType::Tgetattr) => Fcall::Tgetattr(Tgetattr {
                fid: decode!(buf),
                req_mask: decode!(GetattrMask, buf),
            }),
            Some(MsgType::Rgetattr) => {
                let r = Fcall::Rgetattr(Rgetattr {
                    valid: decode!(GetattrMask, buf),
                    qid: decode!(buf),
                    stat: decode!(buf),
                });
                let (_btime, _gen, _ver): (Time, u64, u64) =
                    (decode!(buf), decode!(buf), decode!(buf));
                r
            }
            Some(MsgType::Tsetattr) => Fcall::Tsetattr(Tsetattr {
                fid: decode!(buf),
                valid: decode!(SetattrMask, buf),
                stat: decode!(buf),
            }),
            Some(MsgType::Rsetattr) => Fcall::Rsetattr(Rsetattr {}),
            Some(MsgType::Txattrwalk) => Fcall::Txattrwalk(Txattrwalk {
                fid: decode!(buf),
                newfid: decode!(buf),
                name: decode!(buf),
            }),
            Some(MsgType::Rxattrwalk) => Fcall::Rxattrwalk(Rxattrwalk { size: decode!(buf) }),
            Some(MsgType::Txattrcreate) => Fcall::Txattrcreate(Txattrcreate {
                fid: decode!(buf),
                name: decode!(buf),
                attr_size: decode!(buf),
                flags: decode!(buf),
            }),
            Some(MsgType::Rxattrcreate) => Fcall::Rxattrcreate(Rxattrcreate {}),
            Some(MsgType::Treaddir) => Fcall::Treaddir(Treaddir {
                fid: decode!(buf),
                offset: decode!(buf),
                count: decode!(buf),
            }),
            Some(MsgType::Rreaddir) => Fcall::Rreaddir(Rreaddir { data: decode!(buf) }),
            Some(MsgType::Tfsync) => Fcall::Tfsync(Tfsync { fid: decode!(buf) }),
            Some(MsgType::Rfsync) => Fcall::Rfsync(Rfsync {}),
            Some(MsgType::Tlock) => Fcall::Tlock(Tlock {
                fid: decode!(buf),
                flock: decode!(buf),
            }),
            Some(MsgType::Rlock) => Fcall::Rlock(Rlock {
                status: decode!(LockStatus, buf),
            }),
            Some(MsgType::Tgetlock) => Fcall::Tgetlock(Tgetlock {
                fid: decode!(buf),
                flock: decode!(buf),
            }),
            Some(MsgType::Rgetlock) => Fcall::Rgetlock(Rgetlock {
                flock: decode!(buf),
            }),
            Some(MsgType::Tlink) => Fcall::Tlink(Tlink {
                dfid: decode!(buf),
                fid: decode!(buf),
                name: decode!(buf),
            }),
            Some(MsgType::Rlink) => Fcall::Rlink(Rlink {}),
            Some(MsgType::Tmkdir) => Fcall::Tmkdir(Tmkdir {
                dfid: decode!(buf),
                name: decode!(buf),
                mode: decode!(buf),
                gid: decode!(buf),
            }),
            Some(MsgType::Rmkdir) => Fcall::Rmkdir(Rmkdir { qid: decode!(buf) }),
            Some(MsgType::Trenameat) => Fcall::Trenameat(Trenameat {
                olddirfid: decode!(buf),
                oldname: decode!(buf),
                newdirfid: decode!(buf),
                newname: decode!(buf),
            }),
            Some(MsgType::Rrenameat) => Fcall::Rrenameat(Rrenameat {}),
            Some(MsgType::Tunlinkat) => Fcall::Tunlinkat(Tunlinkat {
                dirfd: decode!(buf),
                name: decode!(buf),
                flags: decode!(buf),
            }),
            Some(MsgType::Runlinkat) => Fcall::Runlinkat(Runlinkat {}),
            Some(MsgType::Tauth) => Fcall::Tauth(Tauth {
                afid: decode!(buf),
                uname: decode!(buf),
                aname: decode!(buf),
                n_uname: decode!(buf),
            }),
            Some(MsgType::Rauth) => Fcall::Rauth(Rauth { aqid: decode!(buf) }),
            Some(MsgType::Tattach) => Fcall::Tattach(Tattach {
                fid: decode!(buf),
                afid: decode!(buf),
                uname: decode!(buf),
                aname: decode!(buf),
                n_uname: decode!(buf),
            }),
            Some(MsgType::Rattach) => Fcall::Rattach(Rattach { qid: decode!(buf) }),
            Some(MsgType::Tversion) => Fcall::Tversion(Tversion {
                msize: decode!(buf),
                version: decode!(buf),
            }),
            Some(MsgType::Rversion) => Fcall::Rversion(Rversion {
                msize: decode!(buf),
                version: decode!(buf),
            }),
            Some(MsgType::Tflush) => Fcall::Tflush(Tflush {
                oldtag: decode!(buf),
            }),
            Some(MsgType::Rflush) => Fcall::Rflush(Rflush {}),
            Some(MsgType::Twalk) => Fcall::Twalk(Twalk {
                fid: decode!(buf),
                newfid: decode!(buf),
                wnames: decode!(buf),
            }),
            Some(MsgType::Rwalk) => Fcall::Rwalk(Rwalk {
                wqids: decode!(buf),
            }),
            Some(MsgType::Tread) => Fcall::Tread(Tread {
                fid: decode!(buf),
                offset: decode!(buf),
                count: decode!(buf),
            }),
            Some(MsgType::Rread) => Fcall::Rread(Rread { data: decode!(buf) }),
            Some(MsgType::Twrite) => Fcall::Twrite(Twrite {
                fid: decode!(buf),
                offset: decode!(buf),
                data: decode!(buf),
            }),
            Some(MsgType::Rwrite) => Fcall::Rwrite(Rwrite {
                count: decode!(buf),
            }),
            Some(MsgType::Tclunk) => Fcall::Tclunk(Tclunk { fid: decode!(buf) }),
            Some(MsgType::Rclunk) => Fcall::Rclunk(Rclunk {}),
            Some(MsgType::Tremove) => Fcall::Tremove(Tremove { fid: decode!(buf) }),
            Some(MsgType::Rremove) => Fcall::Rremove(Rremove {}),
            Some(MsgType::Tlerror) | None => return Err(io_err!(Other, "Invalid message type")),
        };

        Ok(Msg { tag, body })
    }
}

/// Helper function to read a 9P message from a byte-oriented stream
pub fn read_msg<R: ReadBytesExt>(r: &mut R) -> std::io::Result<Msg> {
    Decodable::decode(r)
}

/// Helper function to write a 9P message into a byte-oriented stream
pub fn write_msg<W: WriteBytesExt>(w: &mut W, msg: &Msg) -> std::io::Result<usize> {
    msg.encode(w)
}
