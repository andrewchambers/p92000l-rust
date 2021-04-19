//! 9P protocol data types and constants.
//!
//! # Protocol
//! 9P2000.L

use bitflags::bitflags;
use enum_primitive::*;
use std::convert::TryInto;
use std::fs;
use std::io::{Read, Write};
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

impl<'b> NcDirEntry<'b> {
    pub fn size(&self) -> u32 {
        (13 + 8 + 1 + 2 + self.name.len()) as u32
    }
}

/// Directory entry array
#[derive(Clone, Debug)]
pub struct NcDirEntryData<'b> {
    pub data: Vec<NcDirEntry<'b>>,
}

impl<'b> NcDirEntryData<'b> {
    pub fn new() -> NcDirEntryData<'b> {
        Self::with(Vec::new())
    }
    pub fn with(v: Vec<NcDirEntry<'b>>) -> NcDirEntryData<'b> {
        NcDirEntryData { data: v }
    }
    pub fn data(&self) -> &[NcDirEntry] {
        &self.data
    }
    pub fn size(&self) -> u32 {
        self.data.iter().fold(0, |a, e| a + e.size()) as u32
    }
    pub fn push(&mut self, entry: NcDirEntry<'b>) {
        self.data.push(entry);
    }
}

impl<'b> Default for NcDirEntryData<'b> {
    fn default() -> Self {
        Self::new()
    }
}

impl DirEntry {
    pub fn size(&self) -> u32 {
        (13 + 8 + 1 + 2 + self.name.len()) as u32
    }
}

/// Directory entry array
#[derive(Clone, Debug)]
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

impl Default for DirEntryData {
    fn default() -> Self {
        Self::new()
    }
}

impl<'b> From<NcDirEntryData<'b>> for DirEntryData {
    fn from(v: NcDirEntryData<'b>) -> DirEntryData {
        DirEntryData {
            data: v.data.iter().map(|e| e.into()).collect(),
        }
    }
}

// Commented out the types not used in 9P2000.L
enum_from_primitive! {
    #[doc = "Message type, 9P operations"]
    #[derive(Copy, Clone, Debug)]
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
            _ => EIO,
        };

        Rlerror { ecode }
    }
}

impl<A: Into<Fcall>, B: Into<Fcall>> From<Result<A, B>> for Fcall {
    fn from(r: Result<A, B>) -> Fcall {
        match r {
            Ok(a) => a.into(),
            Err(b) => b.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Msg {
    pub tag: u16,
    pub body: Fcall,
}

#[derive(Clone, Debug)]
pub struct NcMsg<'b> {
    pub tag: u16,
    pub body: NcFcall<'b>,
}

pub fn read_msg<'a, R: Read>(r: &'a mut R, buf: &'a mut Vec<u8>) -> std::io::Result<NcMsg<'a>> {
    let mut sz = [0; 4];
    r.read_exact(&mut sz[..])?;
    let sz = u32::from_le_bytes(sz) as usize;
    if sz > buf.capacity() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "remote violated protocol size limit",
        ));
    }
    let sz = sz - 4;
    buf.resize(sz, 0);
    r.read_exact(&mut buf[..])?;
    decode_msg(buf)
}

pub fn write_msg<W: Write>(w: &mut W, buf: &mut Vec<u8>, msg: &Msg) -> std::io::Result<()> {
    buf.truncate(0);
    match msg {
        Msg {
            tag,
            body: Fcall::Rread(Rread { ref data }),
        } => {
            // Zero copy Rread path.
            let mut cursor = std::io::Cursor::new(buf);
            let sz = 4 + 1 + 2 + 4 + data.len();
            if sz > 0xffffffff {
                return Err(invalid_9p_msg());
            }
            encode_u32(&mut cursor, &(sz as u32))?;
            encode_u8(&mut cursor, &117)?;
            encode_u16(&mut cursor, &tag)?;
            encode_u32(&mut cursor, &(data.len() as u32))?;
            let buf = cursor.into_inner();
            // XXX vectored write?
            w.write_all(&buf[..])?;
            w.write_all(&data[..])?;
            Ok(())
        }
        /* TODO, Zero copy Twrite path, mostly for client.
        Msg {
            tag,
            body: Fcall::Twrite(Twrite { ref data }),
        } => {
        }
        */
        msg => {
            // Slow path, encode the whole message to the buffer then write it.
            let mut cursor = std::io::Cursor::new(buf);
            encode_msg(&mut cursor, msg)?;
            let buf = cursor.into_inner();
            // XXX vectored write or single write here?
            let sz_bytes = &((buf.len() + 4) as u32).to_le_bytes()[..];
            w.write_all(sz_bytes)?;
            w.write_all(&buf[..])?;
            Ok(())
        }
    }
}

struct Decoder<'b> {
    buf: &'b [u8],
}

pub fn decode_msg(buf: &[u8]) -> std::io::Result<NcMsg> {
    let mut d = Decoder { buf };
    d.decode_nc_msg()
}

fn invalid_9p_msg() -> std::io::Error {
    std::io::Error::new(::std::io::ErrorKind::InvalidInput, "invalid 9p message")
}

fn encode_u8<W: Write>(w: &mut W, v: &u8) -> std::io::Result<()> {
    w.write_all(&[*v])?;
    Ok(())
}

fn encode_u16<W: Write>(w: &mut W, v: &u16) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes()[..])?;
    Ok(())
}

fn encode_u32<W: Write>(w: &mut W, v: &u32) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes()[..])?;
    Ok(())
}

fn encode_u64<W: Write>(w: &mut W, v: &u64) -> std::io::Result<()> {
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
    encode_u16(w, &(v.len() as u16))?;
    w.write_all(v.as_bytes())?;
    Ok(())
}

fn encode_string<W: Write>(w: &mut W, v: &str) -> std::io::Result<()> {
    encode_str(w, v)
}

fn encode_data_buf<W: Write>(w: &mut W, v: &[u8]) -> std::io::Result<()> {
    if v.len() > 0xffffffff {
        return Err(std::io::Error::new(
            ::std::io::ErrorKind::InvalidInput,
            "data buf too long for 9p encoding",
        ));
    }
    encode_u32(w, &(v.len() as u32))?;
    w.write_all(v)?;
    Ok(())
}

fn encode_data<W: Write>(w: &mut W, v: &[u8]) -> std::io::Result<()> {
    encode_data_buf(w, v)?;
    Ok(())
}

fn encode_vec_str<W: Write>(w: &mut W, v: &[&str]) -> std::io::Result<()> {
    if v.len() > 0xffff {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "string vec too long for 9p encoding",
        ));
    }
    encode_u16(w, &(v.len() as u16))?;
    for v in v.iter() {
        encode_str(w, v)?;
    }
    Ok(())
}

fn encode_vec_string<W: Write>(w: &mut W, v: &[String]) -> std::io::Result<()> {
    if v.len() > 0xffff {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "string vec too long for 9p encoding",
        ));
    }
    encode_u16(w, &(v.len() as u16))?;
    for v in v.iter() {
        encode_string(w, v)?;
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
    encode_u16(w, &(v.len() as u16))?;
    for v in v.iter() {
        encode_qid(w, v)?;
    }
    Ok(())
}

fn encode_direntrydata<W: Write>(w: &mut W, v: &DirEntryData) -> std::io::Result<()> {
    if v.data.len() > 0xffff {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "dir entry vec too long for encoding",
        ));
    }
    encode_u32(w, &(v.size()))?;
    for v in v.data.iter() {
        encode_direntry(w, v)?;
    }
    Ok(())
}

fn encode_nc_direntrydata<'a, 'b, W: Write>(
    w: &'a mut W,
    v: &'a NcDirEntryData<'b>,
) -> std::io::Result<()> {
    if v.data.len() > 0xffff {
        return Err(std::io::Error::new(
            ::std::io::ErrorKind::InvalidInput,
            "dir entry vec too long for encoding",
        ));
    }
    encode_u16(w, &(v.data.len() as u16))?;
    for v in v.data.iter() {
        encode_nc_direntry(w, v)?;
    }
    Ok(())
}

fn encode_qidtype<W: Write>(w: &mut W, v: &QidType) -> std::io::Result<()> {
    encode_u8(w, &v.bits())
}

fn encode_locktype<W: Write>(w: &mut W, v: &LockType) -> std::io::Result<()> {
    encode_u8(w, &v.bits())
}

fn encode_lockstatus<W: Write>(w: &mut W, v: &LockStatus) -> std::io::Result<()> {
    encode_u8(w, &v.bits())
}

fn encode_lockflag<W: Write>(w: &mut W, v: &LockFlag) -> std::io::Result<()> {
    encode_u32(w, &v.bits())
}

fn encode_getattrmask<W: Write>(w: &mut W, v: &GetattrMask) -> std::io::Result<()> {
    encode_u64(w, &v.bits())
}

fn encode_setattrmask<W: Write>(w: &mut W, v: &SetattrMask) -> std::io::Result<()> {
    encode_u32(w, &v.bits())
}

include!("decoder_impl.gen.rs");
include!("fcall_types.gen.rs");
include!("encoder_impl.gen.rs");
include!("convert_impl.gen.rs");
