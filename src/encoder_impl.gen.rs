#[allow(clippy::all)]
fn encode_qid<W: Write>(w: &mut W, v: &Qid) -> std::io::Result<()> {
    encode_qidtype(w, &v.typ)?;
    encode_u32(w, &v.version)?;
    encode_u64(w, &v.path)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_statfs<W: Write>(w: &mut W, v: &Statfs) -> std::io::Result<()> {
    encode_u32(w, &v.typ)?;
    encode_u32(w, &v.bsize)?;
    encode_u64(w, &v.blocks)?;
    encode_u64(w, &v.bfree)?;
    encode_u64(w, &v.bavail)?;
    encode_u64(w, &v.files)?;
    encode_u64(w, &v.ffree)?;
    encode_u64(w, &v.fsid)?;
    encode_u32(w, &v.namelen)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_time<W: Write>(w: &mut W, v: &Time) -> std::io::Result<()> {
    encode_u64(w, &v.sec)?;
    encode_u64(w, &v.nsec)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_stat<W: Write>(w: &mut W, v: &Stat) -> std::io::Result<()> {
    encode_u32(w, &v.mode)?;
    encode_u32(w, &v.uid)?;
    encode_u32(w, &v.gid)?;
    encode_u64(w, &v.nlink)?;
    encode_u64(w, &v.rdev)?;
    encode_u64(w, &v.size)?;
    encode_u64(w, &v.blksize)?;
    encode_u64(w, &v.blocks)?;
    encode_time(w, &v.atime)?;
    encode_time(w, &v.mtime)?;
    encode_time(w, &v.ctime)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_setattr<W: Write>(w: &mut W, v: &SetAttr) -> std::io::Result<()> {
    encode_u32(w, &v.mode)?;
    encode_u32(w, &v.uid)?;
    encode_u32(w, &v.gid)?;
    encode_u64(w, &v.size)?;
    encode_time(w, &v.atime)?;
    encode_time(w, &v.mtime)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_direntry<'a, 'b, W: Write>(w: &'a mut W, v: &NcDirEntry<'b>) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    encode_u64(w, &v.offset)?;
    encode_u8(w, &v.typ)?;
    encode_str(w, &v.name)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_direntry<W: Write>(w: &mut W, v: &DirEntry) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    encode_u64(w, &v.offset)?;
    encode_u8(w, &v.typ)?;
    encode_string(w, &v.name)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_flock<'a, 'b, W: Write>(w: &'a mut W, v: &NcFlock<'b>) -> std::io::Result<()> {
    encode_locktype(w, &v.typ)?;
    encode_lockflag(w, &v.flags)?;
    encode_u64(w, &v.start)?;
    encode_u64(w, &v.length)?;
    encode_u32(w, &v.proc_id)?;
    encode_str(w, &v.client_id)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_flock<W: Write>(w: &mut W, v: &Flock) -> std::io::Result<()> {
    encode_locktype(w, &v.typ)?;
    encode_lockflag(w, &v.flags)?;
    encode_u64(w, &v.start)?;
    encode_u64(w, &v.length)?;
    encode_u32(w, &v.proc_id)?;
    encode_string(w, &v.client_id)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_getlock<'a, 'b, W: Write>(w: &'a mut W, v: &NcGetlock<'b>) -> std::io::Result<()> {
    encode_locktype(w, &v.typ)?;
    encode_u64(w, &v.start)?;
    encode_u64(w, &v.length)?;
    encode_u32(w, &v.proc_id)?;
    encode_str(w, &v.client_id)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_getlock<W: Write>(w: &mut W, v: &Getlock) -> std::io::Result<()> {
    encode_locktype(w, &v.typ)?;
    encode_u64(w, &v.start)?;
    encode_u64(w, &v.length)?;
    encode_u32(w, &v.proc_id)?;
    encode_string(w, &v.client_id)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rlerror<W: Write>(w: &mut W, v: &Rlerror) -> std::io::Result<()> {
    encode_u32(w, &v.ecode)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tattach<'a, 'b, W: Write>(w: &'a mut W, v: &NcTattach<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.afid)?;
    encode_str(w, &v.uname)?;
    encode_str(w, &v.aname)?;
    encode_u32(w, &v.n_uname)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tattach<W: Write>(w: &mut W, v: &Tattach) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.afid)?;
    encode_string(w, &v.uname)?;
    encode_string(w, &v.aname)?;
    encode_u32(w, &v.n_uname)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rattach<W: Write>(w: &mut W, v: &Rattach) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tstatfs<W: Write>(w: &mut W, v: &Tstatfs) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rstatfs<W: Write>(w: &mut W, v: &Rstatfs) -> std::io::Result<()> {
    encode_statfs(w, &v.statfs)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tlopen<W: Write>(w: &mut W, v: &Tlopen) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.flags)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rlopen<W: Write>(w: &mut W, v: &Rlopen) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    encode_u32(w, &v.iounit)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tlcreate<'a, 'b, W: Write>(w: &'a mut W, v: &NcTlcreate<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_str(w, &v.name)?;
    encode_u32(w, &v.flags)?;
    encode_u32(w, &v.mode)?;
    encode_u32(w, &v.gid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tlcreate<W: Write>(w: &mut W, v: &Tlcreate) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_string(w, &v.name)?;
    encode_u32(w, &v.flags)?;
    encode_u32(w, &v.mode)?;
    encode_u32(w, &v.gid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rlcreate<W: Write>(w: &mut W, v: &Rlcreate) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    encode_u32(w, &v.iounit)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tsymlink<'a, 'b, W: Write>(w: &'a mut W, v: &NcTsymlink<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_str(w, &v.name)?;
    encode_str(w, &v.symtgt)?;
    encode_u32(w, &v.gid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tsymlink<W: Write>(w: &mut W, v: &Tsymlink) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_string(w, &v.name)?;
    encode_string(w, &v.symtgt)?;
    encode_u32(w, &v.gid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rsymlink<W: Write>(w: &mut W, v: &Rsymlink) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tmknod<'a, 'b, W: Write>(w: &'a mut W, v: &NcTmknod<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.dfid)?;
    encode_str(w, &v.name)?;
    encode_u32(w, &v.mode)?;
    encode_u32(w, &v.major)?;
    encode_u32(w, &v.minor)?;
    encode_u32(w, &v.gid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tmknod<W: Write>(w: &mut W, v: &Tmknod) -> std::io::Result<()> {
    encode_u32(w, &v.dfid)?;
    encode_string(w, &v.name)?;
    encode_u32(w, &v.mode)?;
    encode_u32(w, &v.major)?;
    encode_u32(w, &v.minor)?;
    encode_u32(w, &v.gid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rmknod<W: Write>(w: &mut W, v: &Rmknod) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_trename<'a, 'b, W: Write>(w: &'a mut W, v: &NcTrename<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.dfid)?;
    encode_str(w, &v.name)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_trename<W: Write>(w: &mut W, v: &Trename) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.dfid)?;
    encode_string(w, &v.name)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rrename<W: Write>(_w: &mut W, _v: &Rrename) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_treadlink<W: Write>(w: &mut W, v: &Treadlink) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_rreadlink<'a, 'b, W: Write>(w: &'a mut W, v: &NcRreadlink<'b>) -> std::io::Result<()> {
    encode_str(w, &v.target)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rreadlink<W: Write>(w: &mut W, v: &Rreadlink) -> std::io::Result<()> {
    encode_string(w, &v.target)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tgetattr<W: Write>(w: &mut W, v: &Tgetattr) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_getattrmask(w, &v.req_mask)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rgetattr<W: Write>(w: &mut W, v: &Rgetattr) -> std::io::Result<()> {
    encode_getattrmask(w, &v.valid)?;
    encode_qid(w, &v.qid)?;
    encode_stat(w, &v.stat)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tsetattr<W: Write>(w: &mut W, v: &Tsetattr) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_setattrmask(w, &v.valid)?;
    encode_setattr(w, &v.stat)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rsetattr<W: Write>(_w: &mut W, _v: &Rsetattr) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_txattrwalk<'a, 'b, W: Write>(
    w: &'a mut W,
    v: &NcTxattrwalk<'b>,
) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.newfid)?;
    encode_str(w, &v.name)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_txattrwalk<W: Write>(w: &mut W, v: &Txattrwalk) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.newfid)?;
    encode_string(w, &v.name)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rxattrwalk<W: Write>(w: &mut W, v: &Rxattrwalk) -> std::io::Result<()> {
    encode_u64(w, &v.size)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_txattrcreate<'a, 'b, W: Write>(
    w: &'a mut W,
    v: &NcTxattrcreate<'b>,
) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_str(w, &v.name)?;
    encode_u64(w, &v.attr_size)?;
    encode_u32(w, &v.flags)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_txattrcreate<W: Write>(w: &mut W, v: &Txattrcreate) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_string(w, &v.name)?;
    encode_u64(w, &v.attr_size)?;
    encode_u32(w, &v.flags)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rxattrcreate<W: Write>(_w: &mut W, _v: &Rxattrcreate) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_treaddir<W: Write>(w: &mut W, v: &Treaddir) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u64(w, &v.offset)?;
    encode_u32(w, &v.count)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_rreaddir<'a, 'b, W: Write>(w: &'a mut W, v: &NcRreaddir<'b>) -> std::io::Result<()> {
    encode_nc_direntrydata(w, &v.data)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rreaddir<W: Write>(w: &mut W, v: &Rreaddir) -> std::io::Result<()> {
    encode_direntrydata(w, &v.data)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tfsync<W: Write>(w: &mut W, v: &Tfsync) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rfsync<W: Write>(_w: &mut W, _v: &Rfsync) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tlock<'a, 'b, W: Write>(w: &'a mut W, v: &NcTlock<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_nc_flock(w, &v.flock)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tlock<W: Write>(w: &mut W, v: &Tlock) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_flock(w, &v.flock)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rlock<W: Write>(w: &mut W, v: &Rlock) -> std::io::Result<()> {
    encode_lockstatus(w, &v.status)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tgetlock<'a, 'b, W: Write>(w: &'a mut W, v: &NcTgetlock<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_nc_getlock(w, &v.flock)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tgetlock<W: Write>(w: &mut W, v: &Tgetlock) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_getlock(w, &v.flock)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_rgetlock<'a, 'b, W: Write>(w: &'a mut W, v: &NcRgetlock<'b>) -> std::io::Result<()> {
    encode_nc_getlock(w, &v.flock)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rgetlock<W: Write>(w: &mut W, v: &Rgetlock) -> std::io::Result<()> {
    encode_getlock(w, &v.flock)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tlink<'a, 'b, W: Write>(w: &'a mut W, v: &NcTlink<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.dfid)?;
    encode_u32(w, &v.fid)?;
    encode_str(w, &v.name)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tlink<W: Write>(w: &mut W, v: &Tlink) -> std::io::Result<()> {
    encode_u32(w, &v.dfid)?;
    encode_u32(w, &v.fid)?;
    encode_string(w, &v.name)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rlink<W: Write>(_w: &mut W, _v: &Rlink) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tmkdir<'a, 'b, W: Write>(w: &'a mut W, v: &NcTmkdir<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.dfid)?;
    encode_str(w, &v.name)?;
    encode_u32(w, &v.mode)?;
    encode_u32(w, &v.gid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tmkdir<W: Write>(w: &mut W, v: &Tmkdir) -> std::io::Result<()> {
    encode_u32(w, &v.dfid)?;
    encode_string(w, &v.name)?;
    encode_u32(w, &v.mode)?;
    encode_u32(w, &v.gid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rmkdir<W: Write>(w: &mut W, v: &Rmkdir) -> std::io::Result<()> {
    encode_qid(w, &v.qid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_trenameat<'a, 'b, W: Write>(w: &'a mut W, v: &NcTrenameat<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.olddirfid)?;
    encode_str(w, &v.oldname)?;
    encode_u32(w, &v.newdirfid)?;
    encode_str(w, &v.newname)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_trenameat<W: Write>(w: &mut W, v: &Trenameat) -> std::io::Result<()> {
    encode_u32(w, &v.olddirfid)?;
    encode_string(w, &v.oldname)?;
    encode_u32(w, &v.newdirfid)?;
    encode_string(w, &v.newname)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rrenameat<W: Write>(_w: &mut W, _v: &Rrenameat) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tunlinkat<'a, 'b, W: Write>(w: &'a mut W, v: &NcTunlinkat<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.dirfd)?;
    encode_str(w, &v.name)?;
    encode_u32(w, &v.flags)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tunlinkat<W: Write>(w: &mut W, v: &Tunlinkat) -> std::io::Result<()> {
    encode_u32(w, &v.dirfd)?;
    encode_string(w, &v.name)?;
    encode_u32(w, &v.flags)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_runlinkat<W: Write>(_w: &mut W, _v: &Runlinkat) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tauth<'a, 'b, W: Write>(w: &'a mut W, v: &NcTauth<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.afid)?;
    encode_str(w, &v.uname)?;
    encode_str(w, &v.aname)?;
    encode_u32(w, &v.n_uname)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tauth<W: Write>(w: &mut W, v: &Tauth) -> std::io::Result<()> {
    encode_u32(w, &v.afid)?;
    encode_string(w, &v.uname)?;
    encode_string(w, &v.aname)?;
    encode_u32(w, &v.n_uname)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rauth<W: Write>(w: &mut W, v: &Rauth) -> std::io::Result<()> {
    encode_qid(w, &v.aqid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_tversion<'a, 'b, W: Write>(w: &'a mut W, v: &NcTversion<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.msize)?;
    encode_str(w, &v.version)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tversion<W: Write>(w: &mut W, v: &Tversion) -> std::io::Result<()> {
    encode_u32(w, &v.msize)?;
    encode_string(w, &v.version)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_rversion<'a, 'b, W: Write>(w: &'a mut W, v: &NcRversion<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.msize)?;
    encode_str(w, &v.version)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rversion<W: Write>(w: &mut W, v: &Rversion) -> std::io::Result<()> {
    encode_u32(w, &v.msize)?;
    encode_string(w, &v.version)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tflush<W: Write>(w: &mut W, v: &Tflush) -> std::io::Result<()> {
    encode_u16(w, &v.oldtag)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rflush<W: Write>(_w: &mut W, _v: &Rflush) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_twalk<'a, 'b, W: Write>(w: &'a mut W, v: &NcTwalk<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.newfid)?;
    encode_vec_str(w, &v.wnames)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_twalk<W: Write>(w: &mut W, v: &Twalk) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u32(w, &v.newfid)?;
    encode_vec_string(w, &v.wnames)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rwalk<W: Write>(w: &mut W, v: &Rwalk) -> std::io::Result<()> {
    encode_vec_qid(w, &v.wqids)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tread<W: Write>(w: &mut W, v: &Tread) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u64(w, &v.offset)?;
    encode_u32(w, &v.count)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_rread<'a, 'b, W: Write>(w: &'a mut W, v: &NcRread<'b>) -> std::io::Result<()> {
    encode_data_buf(w, &v.data)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rread<W: Write>(w: &mut W, v: &Rread) -> std::io::Result<()> {
    encode_data(w, &v.data)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_nc_twrite<'a, 'b, W: Write>(w: &'a mut W, v: &NcTwrite<'b>) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u64(w, &v.offset)?;
    encode_data_buf(w, &v.data)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_twrite<W: Write>(w: &mut W, v: &Twrite) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    encode_u64(w, &v.offset)?;
    encode_data(w, &v.data)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rwrite<W: Write>(w: &mut W, v: &Rwrite) -> std::io::Result<()> {
    encode_u32(w, &v.count)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_tclunk<W: Write>(w: &mut W, v: &Tclunk) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rclunk<W: Write>(_w: &mut W, _v: &Rclunk) -> std::io::Result<()> {
    Ok(())
}
#[allow(clippy::all)]
fn encode_tremove<W: Write>(w: &mut W, v: &Tremove) -> std::io::Result<()> {
    encode_u32(w, &v.fid)?;
    Ok(())
}
#[allow(clippy::all)]
fn encode_rremove<W: Write>(_w: &mut W, _v: &Rremove) -> std::io::Result<()> {
    Ok(())
}
pub fn encode_nc_msg<W: Write>(w: &mut W, v: &NcMsg) -> std::io::Result<()> {
    let typ = MsgType::from(&v.body);
    encode_u8(w, &(typ as u8))?;
    encode_u16(w, &v.tag)?;
    match v.body {
        NcFcall::Rlerror(ref v) => encode_rlerror(w, v)?,
        NcFcall::Tattach(ref v) => encode_nc_tattach(w, v)?,
        NcFcall::Rattach(ref v) => encode_rattach(w, v)?,
        NcFcall::Tstatfs(ref v) => encode_tstatfs(w, v)?,
        NcFcall::Rstatfs(ref v) => encode_rstatfs(w, v)?,
        NcFcall::Tlopen(ref v) => encode_tlopen(w, v)?,
        NcFcall::Rlopen(ref v) => encode_rlopen(w, v)?,
        NcFcall::Tlcreate(ref v) => encode_nc_tlcreate(w, v)?,
        NcFcall::Rlcreate(ref v) => encode_rlcreate(w, v)?,
        NcFcall::Tsymlink(ref v) => encode_nc_tsymlink(w, v)?,
        NcFcall::Rsymlink(ref v) => encode_rsymlink(w, v)?,
        NcFcall::Tmknod(ref v) => encode_nc_tmknod(w, v)?,
        NcFcall::Rmknod(ref v) => encode_rmknod(w, v)?,
        NcFcall::Trename(ref v) => encode_nc_trename(w, v)?,
        NcFcall::Rrename(ref v) => encode_rrename(w, v)?,
        NcFcall::Treadlink(ref v) => encode_treadlink(w, v)?,
        NcFcall::Rreadlink(ref v) => encode_nc_rreadlink(w, v)?,
        NcFcall::Tgetattr(ref v) => encode_tgetattr(w, v)?,
        NcFcall::Rgetattr(ref v) => encode_rgetattr(w, v)?,
        NcFcall::Tsetattr(ref v) => encode_tsetattr(w, v)?,
        NcFcall::Rsetattr(ref v) => encode_rsetattr(w, v)?,
        NcFcall::Txattrwalk(ref v) => encode_nc_txattrwalk(w, v)?,
        NcFcall::Rxattrwalk(ref v) => encode_rxattrwalk(w, v)?,
        NcFcall::Txattrcreate(ref v) => encode_nc_txattrcreate(w, v)?,
        NcFcall::Rxattrcreate(ref v) => encode_rxattrcreate(w, v)?,
        NcFcall::Treaddir(ref v) => encode_treaddir(w, v)?,
        NcFcall::Rreaddir(ref v) => encode_nc_rreaddir(w, v)?,
        NcFcall::Tfsync(ref v) => encode_tfsync(w, v)?,
        NcFcall::Rfsync(ref v) => encode_rfsync(w, v)?,
        NcFcall::Tlock(ref v) => encode_nc_tlock(w, v)?,
        NcFcall::Rlock(ref v) => encode_rlock(w, v)?,
        NcFcall::Tgetlock(ref v) => encode_nc_tgetlock(w, v)?,
        NcFcall::Rgetlock(ref v) => encode_nc_rgetlock(w, v)?,
        NcFcall::Tlink(ref v) => encode_nc_tlink(w, v)?,
        NcFcall::Rlink(ref v) => encode_rlink(w, v)?,
        NcFcall::Tmkdir(ref v) => encode_nc_tmkdir(w, v)?,
        NcFcall::Rmkdir(ref v) => encode_rmkdir(w, v)?,
        NcFcall::Trenameat(ref v) => encode_nc_trenameat(w, v)?,
        NcFcall::Rrenameat(ref v) => encode_rrenameat(w, v)?,
        NcFcall::Tunlinkat(ref v) => encode_nc_tunlinkat(w, v)?,
        NcFcall::Runlinkat(ref v) => encode_runlinkat(w, v)?,
        NcFcall::Tauth(ref v) => encode_nc_tauth(w, v)?,
        NcFcall::Rauth(ref v) => encode_rauth(w, v)?,
        NcFcall::Tversion(ref v) => encode_nc_tversion(w, v)?,
        NcFcall::Rversion(ref v) => encode_nc_rversion(w, v)?,
        NcFcall::Tflush(ref v) => encode_tflush(w, v)?,
        NcFcall::Rflush(ref v) => encode_rflush(w, v)?,
        NcFcall::Twalk(ref v) => encode_nc_twalk(w, v)?,
        NcFcall::Rwalk(ref v) => encode_rwalk(w, v)?,
        NcFcall::Tread(ref v) => encode_tread(w, v)?,
        NcFcall::Rread(ref v) => encode_nc_rread(w, v)?,
        NcFcall::Twrite(ref v) => encode_nc_twrite(w, v)?,
        NcFcall::Rwrite(ref v) => encode_rwrite(w, v)?,
        NcFcall::Tclunk(ref v) => encode_tclunk(w, v)?,
        NcFcall::Rclunk(ref v) => encode_rclunk(w, v)?,
        NcFcall::Tremove(ref v) => encode_tremove(w, v)?,
        NcFcall::Rremove(ref v) => encode_rremove(w, v)?,
    };
    Ok(())
}
pub fn encode_msg<W: Write>(w: &mut W, v: &Msg) -> std::io::Result<()> {
    let typ = MsgType::from(&v.body);
    encode_u8(w, &(typ as u8))?;
    encode_u16(w, &v.tag)?;
    match v.body {
        Fcall::Rlerror(ref v) => encode_rlerror(w, v)?,
        Fcall::Tattach(ref v) => encode_tattach(w, v)?,
        Fcall::Rattach(ref v) => encode_rattach(w, v)?,
        Fcall::Tstatfs(ref v) => encode_tstatfs(w, v)?,
        Fcall::Rstatfs(ref v) => encode_rstatfs(w, v)?,
        Fcall::Tlopen(ref v) => encode_tlopen(w, v)?,
        Fcall::Rlopen(ref v) => encode_rlopen(w, v)?,
        Fcall::Tlcreate(ref v) => encode_tlcreate(w, v)?,
        Fcall::Rlcreate(ref v) => encode_rlcreate(w, v)?,
        Fcall::Tsymlink(ref v) => encode_tsymlink(w, v)?,
        Fcall::Rsymlink(ref v) => encode_rsymlink(w, v)?,
        Fcall::Tmknod(ref v) => encode_tmknod(w, v)?,
        Fcall::Rmknod(ref v) => encode_rmknod(w, v)?,
        Fcall::Trename(ref v) => encode_trename(w, v)?,
        Fcall::Rrename(ref v) => encode_rrename(w, v)?,
        Fcall::Treadlink(ref v) => encode_treadlink(w, v)?,
        Fcall::Rreadlink(ref v) => encode_rreadlink(w, v)?,
        Fcall::Tgetattr(ref v) => encode_tgetattr(w, v)?,
        Fcall::Rgetattr(ref v) => encode_rgetattr(w, v)?,
        Fcall::Tsetattr(ref v) => encode_tsetattr(w, v)?,
        Fcall::Rsetattr(ref v) => encode_rsetattr(w, v)?,
        Fcall::Txattrwalk(ref v) => encode_txattrwalk(w, v)?,
        Fcall::Rxattrwalk(ref v) => encode_rxattrwalk(w, v)?,
        Fcall::Txattrcreate(ref v) => encode_txattrcreate(w, v)?,
        Fcall::Rxattrcreate(ref v) => encode_rxattrcreate(w, v)?,
        Fcall::Treaddir(ref v) => encode_treaddir(w, v)?,
        Fcall::Rreaddir(ref v) => encode_rreaddir(w, v)?,
        Fcall::Tfsync(ref v) => encode_tfsync(w, v)?,
        Fcall::Rfsync(ref v) => encode_rfsync(w, v)?,
        Fcall::Tlock(ref v) => encode_tlock(w, v)?,
        Fcall::Rlock(ref v) => encode_rlock(w, v)?,
        Fcall::Tgetlock(ref v) => encode_tgetlock(w, v)?,
        Fcall::Rgetlock(ref v) => encode_rgetlock(w, v)?,
        Fcall::Tlink(ref v) => encode_tlink(w, v)?,
        Fcall::Rlink(ref v) => encode_rlink(w, v)?,
        Fcall::Tmkdir(ref v) => encode_tmkdir(w, v)?,
        Fcall::Rmkdir(ref v) => encode_rmkdir(w, v)?,
        Fcall::Trenameat(ref v) => encode_trenameat(w, v)?,
        Fcall::Rrenameat(ref v) => encode_rrenameat(w, v)?,
        Fcall::Tunlinkat(ref v) => encode_tunlinkat(w, v)?,
        Fcall::Runlinkat(ref v) => encode_runlinkat(w, v)?,
        Fcall::Tauth(ref v) => encode_tauth(w, v)?,
        Fcall::Rauth(ref v) => encode_rauth(w, v)?,
        Fcall::Tversion(ref v) => encode_tversion(w, v)?,
        Fcall::Rversion(ref v) => encode_rversion(w, v)?,
        Fcall::Tflush(ref v) => encode_tflush(w, v)?,
        Fcall::Rflush(ref v) => encode_rflush(w, v)?,
        Fcall::Twalk(ref v) => encode_twalk(w, v)?,
        Fcall::Rwalk(ref v) => encode_rwalk(w, v)?,
        Fcall::Tread(ref v) => encode_tread(w, v)?,
        Fcall::Rread(ref v) => encode_rread(w, v)?,
        Fcall::Twrite(ref v) => encode_twrite(w, v)?,
        Fcall::Rwrite(ref v) => encode_rwrite(w, v)?,
        Fcall::Tclunk(ref v) => encode_tclunk(w, v)?,
        Fcall::Rclunk(ref v) => encode_rclunk(w, v)?,
        Fcall::Tremove(ref v) => encode_tremove(w, v)?,
        Fcall::Rremove(ref v) => encode_rremove(w, v)?,
    };
    Ok(())
}
