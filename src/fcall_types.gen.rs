#[derive(Clone, Debug)]
pub struct Qid {
    pub typ: QidType,
    pub version: u32,
    pub path: u64,
}
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct Time {
    pub sec: u64,
    pub nsec: u64,
}
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct SetAttr {
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub size: u64,
    pub atime: Time,
    pub mtime: Time,
}
#[derive(Clone, Debug)]
pub struct NcDirEntry<'b> {
    pub qid: Qid,
    pub offset: u64,
    pub typ: u8,
    pub name: &'b str,
}
#[derive(Clone, Debug)]
pub struct DirEntry {
    pub qid: Qid,
    pub offset: u64,
    pub typ: u8,
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct NcFlock<'b> {
    pub typ: LockType,
    pub flags: LockFlag,
    pub start: u64,
    pub length: u64,
    pub proc_id: u32,
    pub client_id: &'b str,
}
#[derive(Clone, Debug)]
pub struct Flock {
    pub typ: LockType,
    pub flags: LockFlag,
    pub start: u64,
    pub length: u64,
    pub proc_id: u32,
    pub client_id: String,
}
#[derive(Clone, Debug)]
pub struct NcGetlock<'b> {
    pub typ: LockType,
    pub start: u64,
    pub length: u64,
    pub proc_id: u32,
    pub client_id: &'b str,
}
#[derive(Clone, Debug)]
pub struct Getlock {
    pub typ: LockType,
    pub start: u64,
    pub length: u64,
    pub proc_id: u32,
    pub client_id: String,
}
#[derive(Clone, Debug)]
pub struct Rlerror {
    pub ecode: u32,
}
#[derive(Clone, Debug)]
pub struct NcTattach<'b> {
    pub fid: u32,
    pub afid: u32,
    pub uname: &'b str,
    pub aname: &'b str,
    pub n_uname: u32,
}
#[derive(Clone, Debug)]
pub struct Tattach {
    pub fid: u32,
    pub afid: u32,
    pub uname: String,
    pub aname: String,
    pub n_uname: u32,
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
    pub flags: u32,
}
#[derive(Clone, Debug)]
pub struct Rlopen {
    pub qid: Qid,
    pub iounit: u32,
}
#[derive(Clone, Debug)]
pub struct NcTlcreate<'b> {
    pub fid: u32,
    pub name: &'b str,
    pub flags: u32,
    pub mode: u32,
    pub gid: u32,
}
#[derive(Clone, Debug)]
pub struct Tlcreate {
    pub fid: u32,
    pub name: String,
    pub flags: u32,
    pub mode: u32,
    pub gid: u32,
}
#[derive(Clone, Debug)]
pub struct Rlcreate {
    pub qid: Qid,
    pub iounit: u32,
}
#[derive(Clone, Debug)]
pub struct NcTsymlink<'b> {
    pub fid: u32,
    pub name: &'b str,
    pub symtgt: &'b str,
    pub gid: u32,
}
#[derive(Clone, Debug)]
pub struct Tsymlink {
    pub fid: u32,
    pub name: String,
    pub symtgt: String,
    pub gid: u32,
}
#[derive(Clone, Debug)]
pub struct Rsymlink {
    pub qid: Qid,
}
#[derive(Clone, Debug)]
pub struct NcTmknod<'b> {
    pub dfid: u32,
    pub name: &'b str,
    pub mode: u32,
    pub major: u32,
    pub minor: u32,
    pub gid: u32,
}
#[derive(Clone, Debug)]
pub struct Tmknod {
    pub dfid: u32,
    pub name: String,
    pub mode: u32,
    pub major: u32,
    pub minor: u32,
    pub gid: u32,
}
#[derive(Clone, Debug)]
pub struct Rmknod {
    pub qid: Qid,
}
#[derive(Clone, Debug)]
pub struct NcTrename<'b> {
    pub fid: u32,
    pub dfid: u32,
    pub name: &'b str,
}
#[derive(Clone, Debug)]
pub struct Trename {
    pub fid: u32,
    pub dfid: u32,
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct Rrename {}
#[derive(Clone, Debug)]
pub struct Treadlink {
    pub fid: u32,
}
#[derive(Clone, Debug)]
pub struct NcRreadlink<'b> {
    pub target: &'b str,
}
#[derive(Clone, Debug)]
pub struct Rreadlink {
    pub target: String,
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
pub struct NcTxattrwalk<'b> {
    pub fid: u32,
    pub newfid: u32,
    pub name: &'b str,
}
#[derive(Clone, Debug)]
pub struct Txattrwalk {
    pub fid: u32,
    pub newfid: u32,
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct Rxattrwalk {
    pub size: u64,
}
#[derive(Clone, Debug)]
pub struct NcTxattrcreate<'b> {
    pub fid: u32,
    pub name: &'b str,
    pub attr_size: u64,
    pub flags: u32,
}
#[derive(Clone, Debug)]
pub struct Txattrcreate {
    pub fid: u32,
    pub name: String,
    pub attr_size: u64,
    pub flags: u32,
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
pub struct NcRreaddir<'b> {
    pub data: NcDirEntryData<'b>,
}
#[derive(Clone, Debug)]
pub struct Rreaddir {
    pub data: DirEntryData,
}
#[derive(Clone, Debug)]
pub struct Tfsync {
    pub fid: u32,
}
#[derive(Clone, Debug)]
pub struct Rfsync {}
#[derive(Clone, Debug)]
pub struct NcTlock<'b> {
    pub fid: u32,
    pub flock: NcFlock<'b>,
}
#[derive(Clone, Debug)]
pub struct Tlock {
    pub fid: u32,
    pub flock: Flock,
}
#[derive(Clone, Debug)]
pub struct Rlock {
    pub status: LockStatus,
}
#[derive(Clone, Debug)]
pub struct NcTgetlock<'b> {
    pub fid: u32,
    pub flock: NcGetlock<'b>,
}
#[derive(Clone, Debug)]
pub struct Tgetlock {
    pub fid: u32,
    pub flock: Getlock,
}
#[derive(Clone, Debug)]
pub struct NcRgetlock<'b> {
    pub flock: NcGetlock<'b>,
}
#[derive(Clone, Debug)]
pub struct Rgetlock {
    pub flock: Getlock,
}
#[derive(Clone, Debug)]
pub struct NcTlink<'b> {
    pub dfid: u32,
    pub fid: u32,
    pub name: &'b str,
}
#[derive(Clone, Debug)]
pub struct Tlink {
    pub dfid: u32,
    pub fid: u32,
    pub name: String,
}
#[derive(Clone, Debug)]
pub struct Rlink {}
#[derive(Clone, Debug)]
pub struct NcTmkdir<'b> {
    pub dfid: u32,
    pub name: &'b str,
    pub mode: u32,
    pub gid: u32,
}
#[derive(Clone, Debug)]
pub struct Tmkdir {
    pub dfid: u32,
    pub name: String,
    pub mode: u32,
    pub gid: u32,
}
#[derive(Clone, Debug)]
pub struct Rmkdir {
    pub qid: Qid,
}
#[derive(Clone, Debug)]
pub struct NcTrenameat<'b> {
    pub olddirfid: u32,
    pub oldname: &'b str,
    pub newdirfid: u32,
    pub newname: &'b str,
}
#[derive(Clone, Debug)]
pub struct Trenameat {
    pub olddirfid: u32,
    pub oldname: String,
    pub newdirfid: u32,
    pub newname: String,
}
#[derive(Clone, Debug)]
pub struct Rrenameat {}
#[derive(Clone, Debug)]
pub struct NcTunlinkat<'b> {
    pub dirfd: u32,
    pub name: &'b str,
    pub flags: u32,
}
#[derive(Clone, Debug)]
pub struct Tunlinkat {
    pub dirfd: u32,
    pub name: String,
    pub flags: u32,
}
#[derive(Clone, Debug)]
pub struct Runlinkat {}
#[derive(Clone, Debug)]
pub struct NcTauth<'b> {
    pub afid: u32,
    pub uname: &'b str,
    pub aname: &'b str,
    pub n_uname: u32,
}
#[derive(Clone, Debug)]
pub struct Tauth {
    pub afid: u32,
    pub uname: String,
    pub aname: String,
    pub n_uname: u32,
}
#[derive(Clone, Debug)]
pub struct Rauth {
    pub aqid: Qid,
}
#[derive(Clone, Debug)]
pub struct NcTversion<'b> {
    pub msize: u32,
    pub version: &'b str,
}
#[derive(Clone, Debug)]
pub struct Tversion {
    pub msize: u32,
    pub version: String,
}
#[derive(Clone, Debug)]
pub struct NcRversion<'b> {
    pub msize: u32,
    pub version: &'b str,
}
#[derive(Clone, Debug)]
pub struct Rversion {
    pub msize: u32,
    pub version: String,
}
#[derive(Clone, Debug)]
pub struct Tflush {
    pub oldtag: u16,
}
#[derive(Clone, Debug)]
pub struct Rflush {}
#[derive(Clone, Debug)]
pub struct NcTwalk<'b> {
    pub fid: u32,
    pub newfid: u32,
    pub wnames: Vec<&'b str>,
}
#[derive(Clone, Debug)]
pub struct Twalk {
    pub fid: u32,
    pub newfid: u32,
    pub wnames: Vec<String>,
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
pub struct NcRread<'b> {
    pub data: &'b [u8],
}
#[derive(Clone, Debug)]
pub struct Rread {
    pub data: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct NcTwrite<'b> {
    pub fid: u32,
    pub offset: u64,
    pub data: &'b [u8],
}
#[derive(Clone, Debug)]
pub struct Twrite {
    pub fid: u32,
    pub offset: u64,
    pub data: Vec<u8>,
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
impl From<Rlerror> for Fcall {
    fn from(v: Rlerror) -> Fcall {
        Fcall::Rlerror(v)
    }
}
impl From<Tattach> for Fcall {
    fn from(v: Tattach) -> Fcall {
        Fcall::Tattach(v)
    }
}
impl From<Rattach> for Fcall {
    fn from(v: Rattach) -> Fcall {
        Fcall::Rattach(v)
    }
}
impl From<Tstatfs> for Fcall {
    fn from(v: Tstatfs) -> Fcall {
        Fcall::Tstatfs(v)
    }
}
impl From<Rstatfs> for Fcall {
    fn from(v: Rstatfs) -> Fcall {
        Fcall::Rstatfs(v)
    }
}
impl From<Tlopen> for Fcall {
    fn from(v: Tlopen) -> Fcall {
        Fcall::Tlopen(v)
    }
}
impl From<Rlopen> for Fcall {
    fn from(v: Rlopen) -> Fcall {
        Fcall::Rlopen(v)
    }
}
impl From<Tlcreate> for Fcall {
    fn from(v: Tlcreate) -> Fcall {
        Fcall::Tlcreate(v)
    }
}
impl From<Rlcreate> for Fcall {
    fn from(v: Rlcreate) -> Fcall {
        Fcall::Rlcreate(v)
    }
}
impl From<Tsymlink> for Fcall {
    fn from(v: Tsymlink) -> Fcall {
        Fcall::Tsymlink(v)
    }
}
impl From<Rsymlink> for Fcall {
    fn from(v: Rsymlink) -> Fcall {
        Fcall::Rsymlink(v)
    }
}
impl From<Tmknod> for Fcall {
    fn from(v: Tmknod) -> Fcall {
        Fcall::Tmknod(v)
    }
}
impl From<Rmknod> for Fcall {
    fn from(v: Rmknod) -> Fcall {
        Fcall::Rmknod(v)
    }
}
impl From<Trename> for Fcall {
    fn from(v: Trename) -> Fcall {
        Fcall::Trename(v)
    }
}
impl From<Rrename> for Fcall {
    fn from(v: Rrename) -> Fcall {
        Fcall::Rrename(v)
    }
}
impl From<Treadlink> for Fcall {
    fn from(v: Treadlink) -> Fcall {
        Fcall::Treadlink(v)
    }
}
impl From<Rreadlink> for Fcall {
    fn from(v: Rreadlink) -> Fcall {
        Fcall::Rreadlink(v)
    }
}
impl From<Tgetattr> for Fcall {
    fn from(v: Tgetattr) -> Fcall {
        Fcall::Tgetattr(v)
    }
}
impl From<Rgetattr> for Fcall {
    fn from(v: Rgetattr) -> Fcall {
        Fcall::Rgetattr(v)
    }
}
impl From<Tsetattr> for Fcall {
    fn from(v: Tsetattr) -> Fcall {
        Fcall::Tsetattr(v)
    }
}
impl From<Rsetattr> for Fcall {
    fn from(v: Rsetattr) -> Fcall {
        Fcall::Rsetattr(v)
    }
}
impl From<Txattrwalk> for Fcall {
    fn from(v: Txattrwalk) -> Fcall {
        Fcall::Txattrwalk(v)
    }
}
impl From<Rxattrwalk> for Fcall {
    fn from(v: Rxattrwalk) -> Fcall {
        Fcall::Rxattrwalk(v)
    }
}
impl From<Txattrcreate> for Fcall {
    fn from(v: Txattrcreate) -> Fcall {
        Fcall::Txattrcreate(v)
    }
}
impl From<Rxattrcreate> for Fcall {
    fn from(v: Rxattrcreate) -> Fcall {
        Fcall::Rxattrcreate(v)
    }
}
impl From<Treaddir> for Fcall {
    fn from(v: Treaddir) -> Fcall {
        Fcall::Treaddir(v)
    }
}
impl From<Rreaddir> for Fcall {
    fn from(v: Rreaddir) -> Fcall {
        Fcall::Rreaddir(v)
    }
}
impl From<Tfsync> for Fcall {
    fn from(v: Tfsync) -> Fcall {
        Fcall::Tfsync(v)
    }
}
impl From<Rfsync> for Fcall {
    fn from(v: Rfsync) -> Fcall {
        Fcall::Rfsync(v)
    }
}
impl From<Tlock> for Fcall {
    fn from(v: Tlock) -> Fcall {
        Fcall::Tlock(v)
    }
}
impl From<Rlock> for Fcall {
    fn from(v: Rlock) -> Fcall {
        Fcall::Rlock(v)
    }
}
impl From<Tgetlock> for Fcall {
    fn from(v: Tgetlock) -> Fcall {
        Fcall::Tgetlock(v)
    }
}
impl From<Rgetlock> for Fcall {
    fn from(v: Rgetlock) -> Fcall {
        Fcall::Rgetlock(v)
    }
}
impl From<Tlink> for Fcall {
    fn from(v: Tlink) -> Fcall {
        Fcall::Tlink(v)
    }
}
impl From<Rlink> for Fcall {
    fn from(v: Rlink) -> Fcall {
        Fcall::Rlink(v)
    }
}
impl From<Tmkdir> for Fcall {
    fn from(v: Tmkdir) -> Fcall {
        Fcall::Tmkdir(v)
    }
}
impl From<Rmkdir> for Fcall {
    fn from(v: Rmkdir) -> Fcall {
        Fcall::Rmkdir(v)
    }
}
impl From<Trenameat> for Fcall {
    fn from(v: Trenameat) -> Fcall {
        Fcall::Trenameat(v)
    }
}
impl From<Rrenameat> for Fcall {
    fn from(v: Rrenameat) -> Fcall {
        Fcall::Rrenameat(v)
    }
}
impl From<Tunlinkat> for Fcall {
    fn from(v: Tunlinkat) -> Fcall {
        Fcall::Tunlinkat(v)
    }
}
impl From<Runlinkat> for Fcall {
    fn from(v: Runlinkat) -> Fcall {
        Fcall::Runlinkat(v)
    }
}
impl From<Tauth> for Fcall {
    fn from(v: Tauth) -> Fcall {
        Fcall::Tauth(v)
    }
}
impl From<Rauth> for Fcall {
    fn from(v: Rauth) -> Fcall {
        Fcall::Rauth(v)
    }
}
impl From<Tversion> for Fcall {
    fn from(v: Tversion) -> Fcall {
        Fcall::Tversion(v)
    }
}
impl From<Rversion> for Fcall {
    fn from(v: Rversion) -> Fcall {
        Fcall::Rversion(v)
    }
}
impl From<Tflush> for Fcall {
    fn from(v: Tflush) -> Fcall {
        Fcall::Tflush(v)
    }
}
impl From<Rflush> for Fcall {
    fn from(v: Rflush) -> Fcall {
        Fcall::Rflush(v)
    }
}
impl From<Twalk> for Fcall {
    fn from(v: Twalk) -> Fcall {
        Fcall::Twalk(v)
    }
}
impl From<Rwalk> for Fcall {
    fn from(v: Rwalk) -> Fcall {
        Fcall::Rwalk(v)
    }
}
impl From<Tread> for Fcall {
    fn from(v: Tread) -> Fcall {
        Fcall::Tread(v)
    }
}
impl From<Rread> for Fcall {
    fn from(v: Rread) -> Fcall {
        Fcall::Rread(v)
    }
}
impl From<Twrite> for Fcall {
    fn from(v: Twrite) -> Fcall {
        Fcall::Twrite(v)
    }
}
impl From<Rwrite> for Fcall {
    fn from(v: Rwrite) -> Fcall {
        Fcall::Rwrite(v)
    }
}
impl From<Tclunk> for Fcall {
    fn from(v: Tclunk) -> Fcall {
        Fcall::Tclunk(v)
    }
}
impl From<Rclunk> for Fcall {
    fn from(v: Rclunk) -> Fcall {
        Fcall::Rclunk(v)
    }
}
impl From<Tremove> for Fcall {
    fn from(v: Tremove) -> Fcall {
        Fcall::Tremove(v)
    }
}
impl From<Rremove> for Fcall {
    fn from(v: Rremove) -> Fcall {
        Fcall::Rremove(v)
    }
}
#[derive(Clone, Debug)]
pub enum Fcall {
    Rlerror(Rlerror),
    Tattach(Tattach),
    Rattach(Rattach),
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
}
#[derive(Clone, Debug)]
pub enum NcFcall<'b> {
    Rlerror(Rlerror),
    Tattach(NcTattach<'b>),
    Rattach(Rattach),
    Tstatfs(Tstatfs),
    Rstatfs(Rstatfs),
    Tlopen(Tlopen),
    Rlopen(Rlopen),
    Tlcreate(NcTlcreate<'b>),
    Rlcreate(Rlcreate),
    Tsymlink(NcTsymlink<'b>),
    Rsymlink(Rsymlink),
    Tmknod(NcTmknod<'b>),
    Rmknod(Rmknod),
    Trename(NcTrename<'b>),
    Rrename(Rrename),
    Treadlink(Treadlink),
    Rreadlink(NcRreadlink<'b>),
    Tgetattr(Tgetattr),
    Rgetattr(Rgetattr),
    Tsetattr(Tsetattr),
    Rsetattr(Rsetattr),
    Txattrwalk(NcTxattrwalk<'b>),
    Rxattrwalk(Rxattrwalk),
    Txattrcreate(NcTxattrcreate<'b>),
    Rxattrcreate(Rxattrcreate),
    Treaddir(Treaddir),
    Rreaddir(NcRreaddir<'b>),
    Tfsync(Tfsync),
    Rfsync(Rfsync),
    Tlock(NcTlock<'b>),
    Rlock(Rlock),
    Tgetlock(NcTgetlock<'b>),
    Rgetlock(NcRgetlock<'b>),
    Tlink(NcTlink<'b>),
    Rlink(Rlink),
    Tmkdir(NcTmkdir<'b>),
    Rmkdir(Rmkdir),
    Trenameat(NcTrenameat<'b>),
    Rrenameat(Rrenameat),
    Tunlinkat(NcTunlinkat<'b>),
    Runlinkat(Runlinkat),
    Tauth(NcTauth<'b>),
    Rauth(Rauth),
    Tversion(NcTversion<'b>),
    Rversion(NcRversion<'b>),
    Tflush(Tflush),
    Rflush(Rflush),
    Twalk(NcTwalk<'b>),
    Rwalk(Rwalk),
    Tread(Tread),
    Rread(NcRread<'b>),
    Twrite(NcTwrite<'b>),
    Rwrite(Rwrite),
    Tclunk(Tclunk),
    Rclunk(Rclunk),
    Tremove(Tremove),
    Rremove(Rremove),
}
impl<'a> From<&'a Fcall> for MsgType {
    fn from(fcall: &'a Fcall) -> MsgType {
        match *fcall {
            Fcall::Rlerror(_) => MsgType::Rlerror,
            Fcall::Tattach(_) => MsgType::Tattach,
            Fcall::Rattach(_) => MsgType::Rattach,
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
impl<'a, 'b> From<&'a NcFcall<'b>> for MsgType {
    fn from(fcall: &'a NcFcall<'b>) -> MsgType {
        match *fcall {
            NcFcall::Rlerror(_) => MsgType::Rlerror,
            NcFcall::Tattach(_) => MsgType::Tattach,
            NcFcall::Rattach(_) => MsgType::Rattach,
            NcFcall::Tstatfs(_) => MsgType::Tstatfs,
            NcFcall::Rstatfs(_) => MsgType::Rstatfs,
            NcFcall::Tlopen(_) => MsgType::Tlopen,
            NcFcall::Rlopen(_) => MsgType::Rlopen,
            NcFcall::Tlcreate(_) => MsgType::Tlcreate,
            NcFcall::Rlcreate(_) => MsgType::Rlcreate,
            NcFcall::Tsymlink(_) => MsgType::Tsymlink,
            NcFcall::Rsymlink(_) => MsgType::Rsymlink,
            NcFcall::Tmknod(_) => MsgType::Tmknod,
            NcFcall::Rmknod(_) => MsgType::Rmknod,
            NcFcall::Trename(_) => MsgType::Trename,
            NcFcall::Rrename(_) => MsgType::Rrename,
            NcFcall::Treadlink(_) => MsgType::Treadlink,
            NcFcall::Rreadlink(_) => MsgType::Rreadlink,
            NcFcall::Tgetattr(_) => MsgType::Tgetattr,
            NcFcall::Rgetattr(_) => MsgType::Rgetattr,
            NcFcall::Tsetattr(_) => MsgType::Tsetattr,
            NcFcall::Rsetattr(_) => MsgType::Rsetattr,
            NcFcall::Txattrwalk(_) => MsgType::Txattrwalk,
            NcFcall::Rxattrwalk(_) => MsgType::Rxattrwalk,
            NcFcall::Txattrcreate(_) => MsgType::Txattrcreate,
            NcFcall::Rxattrcreate(_) => MsgType::Rxattrcreate,
            NcFcall::Treaddir(_) => MsgType::Treaddir,
            NcFcall::Rreaddir(_) => MsgType::Rreaddir,
            NcFcall::Tfsync(_) => MsgType::Tfsync,
            NcFcall::Rfsync(_) => MsgType::Rfsync,
            NcFcall::Tlock(_) => MsgType::Tlock,
            NcFcall::Rlock(_) => MsgType::Rlock,
            NcFcall::Tgetlock(_) => MsgType::Tgetlock,
            NcFcall::Rgetlock(_) => MsgType::Rgetlock,
            NcFcall::Tlink(_) => MsgType::Tlink,
            NcFcall::Rlink(_) => MsgType::Rlink,
            NcFcall::Tmkdir(_) => MsgType::Tmkdir,
            NcFcall::Rmkdir(_) => MsgType::Rmkdir,
            NcFcall::Trenameat(_) => MsgType::Trenameat,
            NcFcall::Rrenameat(_) => MsgType::Rrenameat,
            NcFcall::Tunlinkat(_) => MsgType::Tunlinkat,
            NcFcall::Runlinkat(_) => MsgType::Runlinkat,
            NcFcall::Tauth(_) => MsgType::Tauth,
            NcFcall::Rauth(_) => MsgType::Rauth,
            NcFcall::Tversion(_) => MsgType::Tversion,
            NcFcall::Rversion(_) => MsgType::Rversion,
            NcFcall::Tflush(_) => MsgType::Tflush,
            NcFcall::Rflush(_) => MsgType::Rflush,
            NcFcall::Twalk(_) => MsgType::Twalk,
            NcFcall::Rwalk(_) => MsgType::Rwalk,
            NcFcall::Tread(_) => MsgType::Tread,
            NcFcall::Rread(_) => MsgType::Rread,
            NcFcall::Twrite(_) => MsgType::Twrite,
            NcFcall::Rwrite(_) => MsgType::Rwrite,
            NcFcall::Tclunk(_) => MsgType::Tclunk,
            NcFcall::Rclunk(_) => MsgType::Rclunk,
            NcFcall::Tremove(_) => MsgType::Tremove,
            NcFcall::Rremove(_) => MsgType::Rremove,
        }
    }
}
