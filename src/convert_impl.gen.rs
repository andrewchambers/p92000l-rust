impl<'b> From<&NcDirEntry<'b>> for DirEntry {
    #[allow(clippy::all)]
    fn from(v: &NcDirEntry<'b>) -> DirEntry {
        DirEntry {
            qid: v.qid.clone(),
            offset: v.offset.clone(),
            typ: v.typ.clone(),
            name: v.name.clone().into(),
        }
    }
}
impl<'b> From<NcDirEntry<'b>> for DirEntry {
    #[allow(clippy::all)]
    fn from(v: NcDirEntry<'b>) -> DirEntry {
        DirEntry {
            qid: v.qid,
            offset: v.offset,
            typ: v.typ,
            name: v.name.into(),
        }
    }
}
impl<'b> From<&NcFlock<'b>> for Flock {
    #[allow(clippy::all)]
    fn from(v: &NcFlock<'b>) -> Flock {
        Flock {
            typ: v.typ.clone(),
            flags: v.flags.clone(),
            start: v.start.clone(),
            length: v.length.clone(),
            proc_id: v.proc_id.clone(),
            client_id: v.client_id.clone().into(),
        }
    }
}
impl<'b> From<NcFlock<'b>> for Flock {
    #[allow(clippy::all)]
    fn from(v: NcFlock<'b>) -> Flock {
        Flock {
            typ: v.typ,
            flags: v.flags,
            start: v.start,
            length: v.length,
            proc_id: v.proc_id,
            client_id: v.client_id.into(),
        }
    }
}
impl<'b> From<&NcGetlock<'b>> for Getlock {
    #[allow(clippy::all)]
    fn from(v: &NcGetlock<'b>) -> Getlock {
        Getlock {
            typ: v.typ.clone(),
            start: v.start.clone(),
            length: v.length.clone(),
            proc_id: v.proc_id.clone(),
            client_id: v.client_id.clone().into(),
        }
    }
}
impl<'b> From<NcGetlock<'b>> for Getlock {
    #[allow(clippy::all)]
    fn from(v: NcGetlock<'b>) -> Getlock {
        Getlock {
            typ: v.typ,
            start: v.start,
            length: v.length,
            proc_id: v.proc_id,
            client_id: v.client_id.into(),
        }
    }
}
impl<'b> From<&NcTattach<'b>> for Tattach {
    #[allow(clippy::all)]
    fn from(v: &NcTattach<'b>) -> Tattach {
        Tattach {
            fid: v.fid.clone(),
            afid: v.afid.clone(),
            uname: v.uname.clone().into(),
            aname: v.aname.clone().into(),
            n_uname: v.n_uname.clone(),
        }
    }
}
impl<'b> From<NcTattach<'b>> for Tattach {
    #[allow(clippy::all)]
    fn from(v: NcTattach<'b>) -> Tattach {
        Tattach {
            fid: v.fid,
            afid: v.afid,
            uname: v.uname.into(),
            aname: v.aname.into(),
            n_uname: v.n_uname,
        }
    }
}
impl<'b> From<&NcTlcreate<'b>> for Tlcreate {
    #[allow(clippy::all)]
    fn from(v: &NcTlcreate<'b>) -> Tlcreate {
        Tlcreate {
            fid: v.fid.clone(),
            name: v.name.clone().into(),
            flags: v.flags.clone(),
            mode: v.mode.clone(),
            gid: v.gid.clone(),
        }
    }
}
impl<'b> From<NcTlcreate<'b>> for Tlcreate {
    #[allow(clippy::all)]
    fn from(v: NcTlcreate<'b>) -> Tlcreate {
        Tlcreate {
            fid: v.fid,
            name: v.name.into(),
            flags: v.flags,
            mode: v.mode,
            gid: v.gid,
        }
    }
}
impl<'b> From<&NcTsymlink<'b>> for Tsymlink {
    #[allow(clippy::all)]
    fn from(v: &NcTsymlink<'b>) -> Tsymlink {
        Tsymlink {
            fid: v.fid.clone(),
            name: v.name.clone().into(),
            symtgt: v.symtgt.clone().into(),
            gid: v.gid.clone(),
        }
    }
}
impl<'b> From<NcTsymlink<'b>> for Tsymlink {
    #[allow(clippy::all)]
    fn from(v: NcTsymlink<'b>) -> Tsymlink {
        Tsymlink {
            fid: v.fid,
            name: v.name.into(),
            symtgt: v.symtgt.into(),
            gid: v.gid,
        }
    }
}
impl<'b> From<&NcTmknod<'b>> for Tmknod {
    #[allow(clippy::all)]
    fn from(v: &NcTmknod<'b>) -> Tmknod {
        Tmknod {
            dfid: v.dfid.clone(),
            name: v.name.clone().into(),
            mode: v.mode.clone(),
            major: v.major.clone(),
            minor: v.minor.clone(),
            gid: v.gid.clone(),
        }
    }
}
impl<'b> From<NcTmknod<'b>> for Tmknod {
    #[allow(clippy::all)]
    fn from(v: NcTmknod<'b>) -> Tmknod {
        Tmknod {
            dfid: v.dfid,
            name: v.name.into(),
            mode: v.mode,
            major: v.major,
            minor: v.minor,
            gid: v.gid,
        }
    }
}
impl<'b> From<&NcTrename<'b>> for Trename {
    #[allow(clippy::all)]
    fn from(v: &NcTrename<'b>) -> Trename {
        Trename {
            fid: v.fid.clone(),
            dfid: v.dfid.clone(),
            name: v.name.clone().into(),
        }
    }
}
impl<'b> From<NcTrename<'b>> for Trename {
    #[allow(clippy::all)]
    fn from(v: NcTrename<'b>) -> Trename {
        Trename {
            fid: v.fid,
            dfid: v.dfid,
            name: v.name.into(),
        }
    }
}
impl<'b> From<&NcRreadlink<'b>> for Rreadlink {
    #[allow(clippy::all)]
    fn from(v: &NcRreadlink<'b>) -> Rreadlink {
        Rreadlink {
            target: v.target.clone().into(),
        }
    }
}
impl<'b> From<NcRreadlink<'b>> for Rreadlink {
    #[allow(clippy::all)]
    fn from(v: NcRreadlink<'b>) -> Rreadlink {
        Rreadlink {
            target: v.target.into(),
        }
    }
}
impl<'b> From<&NcTxattrwalk<'b>> for Txattrwalk {
    #[allow(clippy::all)]
    fn from(v: &NcTxattrwalk<'b>) -> Txattrwalk {
        Txattrwalk {
            fid: v.fid.clone(),
            newfid: v.newfid.clone(),
            name: v.name.clone().into(),
        }
    }
}
impl<'b> From<NcTxattrwalk<'b>> for Txattrwalk {
    #[allow(clippy::all)]
    fn from(v: NcTxattrwalk<'b>) -> Txattrwalk {
        Txattrwalk {
            fid: v.fid,
            newfid: v.newfid,
            name: v.name.into(),
        }
    }
}
impl<'b> From<&NcTxattrcreate<'b>> for Txattrcreate {
    #[allow(clippy::all)]
    fn from(v: &NcTxattrcreate<'b>) -> Txattrcreate {
        Txattrcreate {
            fid: v.fid.clone(),
            name: v.name.clone().into(),
            attr_size: v.attr_size.clone(),
            flags: v.flags.clone(),
        }
    }
}
impl<'b> From<NcTxattrcreate<'b>> for Txattrcreate {
    #[allow(clippy::all)]
    fn from(v: NcTxattrcreate<'b>) -> Txattrcreate {
        Txattrcreate {
            fid: v.fid,
            name: v.name.into(),
            attr_size: v.attr_size,
            flags: v.flags,
        }
    }
}
impl<'b> From<&NcRreaddir<'b>> for Rreaddir {
    #[allow(clippy::all)]
    fn from(v: &NcRreaddir<'b>) -> Rreaddir {
        Rreaddir {
            data: v.data.clone().into(),
        }
    }
}
impl<'b> From<NcRreaddir<'b>> for Rreaddir {
    #[allow(clippy::all)]
    fn from(v: NcRreaddir<'b>) -> Rreaddir {
        Rreaddir {
            data: v.data.into(),
        }
    }
}
impl<'b> From<&NcTlock<'b>> for Tlock {
    #[allow(clippy::all)]
    fn from(v: &NcTlock<'b>) -> Tlock {
        Tlock {
            fid: v.fid.clone(),
            flock: v.flock.clone().into(),
        }
    }
}
impl<'b> From<NcTlock<'b>> for Tlock {
    #[allow(clippy::all)]
    fn from(v: NcTlock<'b>) -> Tlock {
        Tlock {
            fid: v.fid,
            flock: v.flock.into(),
        }
    }
}
impl<'b> From<&NcTgetlock<'b>> for Tgetlock {
    #[allow(clippy::all)]
    fn from(v: &NcTgetlock<'b>) -> Tgetlock {
        Tgetlock {
            fid: v.fid.clone(),
            flock: v.flock.clone().into(),
        }
    }
}
impl<'b> From<NcTgetlock<'b>> for Tgetlock {
    #[allow(clippy::all)]
    fn from(v: NcTgetlock<'b>) -> Tgetlock {
        Tgetlock {
            fid: v.fid,
            flock: v.flock.into(),
        }
    }
}
impl<'b> From<&NcRgetlock<'b>> for Rgetlock {
    #[allow(clippy::all)]
    fn from(v: &NcRgetlock<'b>) -> Rgetlock {
        Rgetlock {
            flock: v.flock.clone().into(),
        }
    }
}
impl<'b> From<NcRgetlock<'b>> for Rgetlock {
    #[allow(clippy::all)]
    fn from(v: NcRgetlock<'b>) -> Rgetlock {
        Rgetlock {
            flock: v.flock.into(),
        }
    }
}
impl<'b> From<&NcTlink<'b>> for Tlink {
    #[allow(clippy::all)]
    fn from(v: &NcTlink<'b>) -> Tlink {
        Tlink {
            dfid: v.dfid.clone(),
            fid: v.fid.clone(),
            name: v.name.clone().into(),
        }
    }
}
impl<'b> From<NcTlink<'b>> for Tlink {
    #[allow(clippy::all)]
    fn from(v: NcTlink<'b>) -> Tlink {
        Tlink {
            dfid: v.dfid,
            fid: v.fid,
            name: v.name.into(),
        }
    }
}
impl<'b> From<&NcTmkdir<'b>> for Tmkdir {
    #[allow(clippy::all)]
    fn from(v: &NcTmkdir<'b>) -> Tmkdir {
        Tmkdir {
            dfid: v.dfid.clone(),
            name: v.name.clone().into(),
            mode: v.mode.clone(),
            gid: v.gid.clone(),
        }
    }
}
impl<'b> From<NcTmkdir<'b>> for Tmkdir {
    #[allow(clippy::all)]
    fn from(v: NcTmkdir<'b>) -> Tmkdir {
        Tmkdir {
            dfid: v.dfid,
            name: v.name.into(),
            mode: v.mode,
            gid: v.gid,
        }
    }
}
impl<'b> From<&NcTrenameat<'b>> for Trenameat {
    #[allow(clippy::all)]
    fn from(v: &NcTrenameat<'b>) -> Trenameat {
        Trenameat {
            olddirfid: v.olddirfid.clone(),
            oldname: v.oldname.clone().into(),
            newdirfid: v.newdirfid.clone(),
            newname: v.newname.clone().into(),
        }
    }
}
impl<'b> From<NcTrenameat<'b>> for Trenameat {
    #[allow(clippy::all)]
    fn from(v: NcTrenameat<'b>) -> Trenameat {
        Trenameat {
            olddirfid: v.olddirfid,
            oldname: v.oldname.into(),
            newdirfid: v.newdirfid,
            newname: v.newname.into(),
        }
    }
}
impl<'b> From<&NcTunlinkat<'b>> for Tunlinkat {
    #[allow(clippy::all)]
    fn from(v: &NcTunlinkat<'b>) -> Tunlinkat {
        Tunlinkat {
            dirfd: v.dirfd.clone(),
            name: v.name.clone().into(),
            flags: v.flags.clone(),
        }
    }
}
impl<'b> From<NcTunlinkat<'b>> for Tunlinkat {
    #[allow(clippy::all)]
    fn from(v: NcTunlinkat<'b>) -> Tunlinkat {
        Tunlinkat {
            dirfd: v.dirfd,
            name: v.name.into(),
            flags: v.flags,
        }
    }
}
impl<'b> From<&NcTauth<'b>> for Tauth {
    #[allow(clippy::all)]
    fn from(v: &NcTauth<'b>) -> Tauth {
        Tauth {
            afid: v.afid.clone(),
            uname: v.uname.clone().into(),
            aname: v.aname.clone().into(),
            n_uname: v.n_uname.clone(),
        }
    }
}
impl<'b> From<NcTauth<'b>> for Tauth {
    #[allow(clippy::all)]
    fn from(v: NcTauth<'b>) -> Tauth {
        Tauth {
            afid: v.afid,
            uname: v.uname.into(),
            aname: v.aname.into(),
            n_uname: v.n_uname,
        }
    }
}
impl<'b> From<&NcTversion<'b>> for Tversion {
    #[allow(clippy::all)]
    fn from(v: &NcTversion<'b>) -> Tversion {
        Tversion {
            msize: v.msize.clone(),
            version: v.version.clone().into(),
        }
    }
}
impl<'b> From<NcTversion<'b>> for Tversion {
    #[allow(clippy::all)]
    fn from(v: NcTversion<'b>) -> Tversion {
        Tversion {
            msize: v.msize,
            version: v.version.into(),
        }
    }
}
impl<'b> From<&NcRversion<'b>> for Rversion {
    #[allow(clippy::all)]
    fn from(v: &NcRversion<'b>) -> Rversion {
        Rversion {
            msize: v.msize.clone(),
            version: v.version.clone().into(),
        }
    }
}
impl<'b> From<NcRversion<'b>> for Rversion {
    #[allow(clippy::all)]
    fn from(v: NcRversion<'b>) -> Rversion {
        Rversion {
            msize: v.msize,
            version: v.version.into(),
        }
    }
}
impl<'b> From<&NcTwalk<'b>> for Twalk {
    #[allow(clippy::all)]
    fn from(v: &NcTwalk<'b>) -> Twalk {
        Twalk {
            fid: v.fid.clone(),
            newfid: v.newfid.clone(),
            wnames: v.wnames.iter().map(|x| x.to_string()).collect(),
        }
    }
}
impl<'b> From<NcTwalk<'b>> for Twalk {
    #[allow(clippy::all)]
    fn from(v: NcTwalk<'b>) -> Twalk {
        Twalk {
            fid: v.fid,
            newfid: v.newfid,
            wnames: v.wnames.iter().map(|x| x.to_string()).collect(),
        }
    }
}
impl<'b> From<&NcRread<'b>> for Rread {
    #[allow(clippy::all)]
    fn from(v: &NcRread<'b>) -> Rread {
        Rread {
            data: v.data.clone().into(),
        }
    }
}
impl<'b> From<NcRread<'b>> for Rread {
    #[allow(clippy::all)]
    fn from(v: NcRread<'b>) -> Rread {
        Rread {
            data: v.data.into(),
        }
    }
}
impl<'b> From<&NcTwrite<'b>> for Twrite {
    #[allow(clippy::all)]
    fn from(v: &NcTwrite<'b>) -> Twrite {
        Twrite {
            fid: v.fid.clone(),
            offset: v.offset.clone(),
            data: v.data.clone().into(),
        }
    }
}
impl<'b> From<NcTwrite<'b>> for Twrite {
    #[allow(clippy::all)]
    fn from(v: NcTwrite<'b>) -> Twrite {
        Twrite {
            fid: v.fid,
            offset: v.offset,
            data: v.data.into(),
        }
    }
}
impl<'a, 'b> From<&'a NcFcall<'b>> for Fcall {
    fn from(fcall: &'a NcFcall<'b>) -> Fcall {
        match fcall {
            NcFcall::Rlerror(ref v) => Fcall::Rlerror(v.clone()),
            NcFcall::Tattach(ref v) => Fcall::Tattach(v.into()),
            NcFcall::Rattach(ref v) => Fcall::Rattach(v.clone()),
            NcFcall::Tstatfs(ref v) => Fcall::Tstatfs(v.clone()),
            NcFcall::Rstatfs(ref v) => Fcall::Rstatfs(v.clone()),
            NcFcall::Tlopen(ref v) => Fcall::Tlopen(v.clone()),
            NcFcall::Rlopen(ref v) => Fcall::Rlopen(v.clone()),
            NcFcall::Tlcreate(ref v) => Fcall::Tlcreate(v.into()),
            NcFcall::Rlcreate(ref v) => Fcall::Rlcreate(v.clone()),
            NcFcall::Tsymlink(ref v) => Fcall::Tsymlink(v.into()),
            NcFcall::Rsymlink(ref v) => Fcall::Rsymlink(v.clone()),
            NcFcall::Tmknod(ref v) => Fcall::Tmknod(v.into()),
            NcFcall::Rmknod(ref v) => Fcall::Rmknod(v.clone()),
            NcFcall::Trename(ref v) => Fcall::Trename(v.into()),
            NcFcall::Rrename(ref v) => Fcall::Rrename(v.clone()),
            NcFcall::Treadlink(ref v) => Fcall::Treadlink(v.clone()),
            NcFcall::Rreadlink(ref v) => Fcall::Rreadlink(v.into()),
            NcFcall::Tgetattr(ref v) => Fcall::Tgetattr(v.clone()),
            NcFcall::Rgetattr(ref v) => Fcall::Rgetattr(v.clone()),
            NcFcall::Tsetattr(ref v) => Fcall::Tsetattr(v.clone()),
            NcFcall::Rsetattr(ref v) => Fcall::Rsetattr(v.clone()),
            NcFcall::Txattrwalk(ref v) => Fcall::Txattrwalk(v.into()),
            NcFcall::Rxattrwalk(ref v) => Fcall::Rxattrwalk(v.clone()),
            NcFcall::Txattrcreate(ref v) => Fcall::Txattrcreate(v.into()),
            NcFcall::Rxattrcreate(ref v) => Fcall::Rxattrcreate(v.clone()),
            NcFcall::Treaddir(ref v) => Fcall::Treaddir(v.clone()),
            NcFcall::Rreaddir(ref v) => Fcall::Rreaddir(v.into()),
            NcFcall::Tfsync(ref v) => Fcall::Tfsync(v.clone()),
            NcFcall::Rfsync(ref v) => Fcall::Rfsync(v.clone()),
            NcFcall::Tlock(ref v) => Fcall::Tlock(v.into()),
            NcFcall::Rlock(ref v) => Fcall::Rlock(v.clone()),
            NcFcall::Tgetlock(ref v) => Fcall::Tgetlock(v.into()),
            NcFcall::Rgetlock(ref v) => Fcall::Rgetlock(v.into()),
            NcFcall::Tlink(ref v) => Fcall::Tlink(v.into()),
            NcFcall::Rlink(ref v) => Fcall::Rlink(v.clone()),
            NcFcall::Tmkdir(ref v) => Fcall::Tmkdir(v.into()),
            NcFcall::Rmkdir(ref v) => Fcall::Rmkdir(v.clone()),
            NcFcall::Trenameat(ref v) => Fcall::Trenameat(v.into()),
            NcFcall::Rrenameat(ref v) => Fcall::Rrenameat(v.clone()),
            NcFcall::Tunlinkat(ref v) => Fcall::Tunlinkat(v.into()),
            NcFcall::Runlinkat(ref v) => Fcall::Runlinkat(v.clone()),
            NcFcall::Tauth(ref v) => Fcall::Tauth(v.into()),
            NcFcall::Rauth(ref v) => Fcall::Rauth(v.clone()),
            NcFcall::Tversion(ref v) => Fcall::Tversion(v.into()),
            NcFcall::Rversion(ref v) => Fcall::Rversion(v.into()),
            NcFcall::Tflush(ref v) => Fcall::Tflush(v.clone()),
            NcFcall::Rflush(ref v) => Fcall::Rflush(v.clone()),
            NcFcall::Twalk(ref v) => Fcall::Twalk(v.into()),
            NcFcall::Rwalk(ref v) => Fcall::Rwalk(v.clone()),
            NcFcall::Tread(ref v) => Fcall::Tread(v.clone()),
            NcFcall::Rread(ref v) => Fcall::Rread(v.into()),
            NcFcall::Twrite(ref v) => Fcall::Twrite(v.into()),
            NcFcall::Rwrite(ref v) => Fcall::Rwrite(v.clone()),
            NcFcall::Tclunk(ref v) => Fcall::Tclunk(v.clone()),
            NcFcall::Rclunk(ref v) => Fcall::Rclunk(v.clone()),
            NcFcall::Tremove(ref v) => Fcall::Tremove(v.clone()),
            NcFcall::Rremove(ref v) => Fcall::Rremove(v.clone()),
        }
    }
}
