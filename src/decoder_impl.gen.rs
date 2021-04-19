impl<'a, 'b: 'a> Decoder<'b> {
    fn decode_u8(&'a mut self) -> std::io::Result<u8> {
        if let Some(v) = self.buf.get(0) {
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

    fn decode_str(&mut self) -> std::io::Result<&'b str> {
        let n = self.decode_u16()? as usize;
        if self.buf.len() >= n {
            match std::str::from_utf8(&self.buf[..n]) {
                Ok(s) => {
                    self.buf = &self.buf[n..];
                    Ok(s)
                }
                Err(_) => Err(invalid_9p_msg()),
            }
        } else {
            Err(invalid_9p_msg())
        }
    }

    fn decode_data_buf(&mut self) -> std::io::Result<&'b [u8]> {
        let n = self.decode_u32()? as usize;
        if self.buf.len() >= n {
            let v = &self.buf[..n];
            self.buf = &self.buf[n..];
            Ok(v)
        } else {
            Err(invalid_9p_msg())
        }
    }

    fn decode_vec_str(&mut self) -> std::io::Result<Vec<&'b str>> {
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

    fn decode_nc_direntrydata(&mut self) -> std::io::Result<NcDirEntryData<'b>> {
        let len = self.decode_u16()?;
        let mut v = Vec::new();
        for _ in 0..len {
            v.push(self.decode_nc_direntry()?);
        }
        Ok(NcDirEntryData::with(v))
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

    #[allow(clippy::all)]
    fn decode_qid(&mut self) -> std::io::Result<Qid> {
        Ok(Qid {
            typ: self.decode_qidtype()?,
            version: self.decode_u32()?,
            path: self.decode_u64()?,
        })
    }
    #[allow(clippy::all)]
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
    #[allow(clippy::all)]
    fn decode_time(&mut self) -> std::io::Result<Time> {
        Ok(Time {
            sec: self.decode_u64()?,
            nsec: self.decode_u64()?,
        })
    }
    #[allow(clippy::all)]
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
    #[allow(clippy::all)]
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
    #[allow(clippy::all)]
    fn decode_nc_direntry(&mut self) -> std::io::Result<NcDirEntry<'b>> {
        Ok(NcDirEntry {
            qid: self.decode_qid()?,
            offset: self.decode_u64()?,
            typ: self.decode_u8()?,
            name: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_flock(&mut self) -> std::io::Result<NcFlock<'b>> {
        Ok(NcFlock {
            typ: self.decode_locktype()?,
            flags: self.decode_lockflag()?,
            start: self.decode_u64()?,
            length: self.decode_u64()?,
            proc_id: self.decode_u32()?,
            client_id: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_getlock(&mut self) -> std::io::Result<NcGetlock<'b>> {
        Ok(NcGetlock {
            typ: self.decode_locktype()?,
            start: self.decode_u64()?,
            length: self.decode_u64()?,
            proc_id: self.decode_u32()?,
            client_id: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rlerror(&mut self) -> std::io::Result<Rlerror> {
        Ok(Rlerror {
            ecode: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_tattach(&mut self) -> std::io::Result<NcTattach<'b>> {
        Ok(NcTattach {
            fid: self.decode_u32()?,
            afid: self.decode_u32()?,
            uname: self.decode_str()?,
            aname: self.decode_str()?,
            n_uname: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rattach(&mut self) -> std::io::Result<Rattach> {
        Ok(Rattach {
            qid: self.decode_qid()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_tstatfs(&mut self) -> std::io::Result<Tstatfs> {
        Ok(Tstatfs {
            fid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rstatfs(&mut self) -> std::io::Result<Rstatfs> {
        Ok(Rstatfs {
            statfs: self.decode_statfs()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_tlopen(&mut self) -> std::io::Result<Tlopen> {
        Ok(Tlopen {
            fid: self.decode_u32()?,
            flags: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rlopen(&mut self) -> std::io::Result<Rlopen> {
        Ok(Rlopen {
            qid: self.decode_qid()?,
            iounit: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_tlcreate(&mut self) -> std::io::Result<NcTlcreate<'b>> {
        Ok(NcTlcreate {
            fid: self.decode_u32()?,
            name: self.decode_str()?,
            flags: self.decode_u32()?,
            mode: self.decode_u32()?,
            gid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rlcreate(&mut self) -> std::io::Result<Rlcreate> {
        Ok(Rlcreate {
            qid: self.decode_qid()?,
            iounit: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_tsymlink(&mut self) -> std::io::Result<NcTsymlink<'b>> {
        Ok(NcTsymlink {
            fid: self.decode_u32()?,
            name: self.decode_str()?,
            symtgt: self.decode_str()?,
            gid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rsymlink(&mut self) -> std::io::Result<Rsymlink> {
        Ok(Rsymlink {
            qid: self.decode_qid()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_tmknod(&mut self) -> std::io::Result<NcTmknod<'b>> {
        Ok(NcTmknod {
            dfid: self.decode_u32()?,
            name: self.decode_str()?,
            mode: self.decode_u32()?,
            major: self.decode_u32()?,
            minor: self.decode_u32()?,
            gid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rmknod(&mut self) -> std::io::Result<Rmknod> {
        Ok(Rmknod {
            qid: self.decode_qid()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_trename(&mut self) -> std::io::Result<NcTrename<'b>> {
        Ok(NcTrename {
            fid: self.decode_u32()?,
            dfid: self.decode_u32()?,
            name: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rrename(&mut self) -> std::io::Result<Rrename> {
        Ok(Rrename {})
    }
    #[allow(clippy::all)]
    fn decode_treadlink(&mut self) -> std::io::Result<Treadlink> {
        Ok(Treadlink {
            fid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_rreadlink(&mut self) -> std::io::Result<NcRreadlink<'b>> {
        Ok(NcRreadlink {
            target: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_tgetattr(&mut self) -> std::io::Result<Tgetattr> {
        Ok(Tgetattr {
            fid: self.decode_u32()?,
            req_mask: self.decode_getattrmask()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rgetattr(&mut self) -> std::io::Result<Rgetattr> {
        Ok(Rgetattr {
            valid: self.decode_getattrmask()?,
            qid: self.decode_qid()?,
            stat: self.decode_stat()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_tsetattr(&mut self) -> std::io::Result<Tsetattr> {
        Ok(Tsetattr {
            fid: self.decode_u32()?,
            valid: self.decode_setattrmask()?,
            stat: self.decode_setattr()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rsetattr(&mut self) -> std::io::Result<Rsetattr> {
        Ok(Rsetattr {})
    }
    #[allow(clippy::all)]
    fn decode_nc_txattrwalk(&mut self) -> std::io::Result<NcTxattrwalk<'b>> {
        Ok(NcTxattrwalk {
            fid: self.decode_u32()?,
            newfid: self.decode_u32()?,
            name: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rxattrwalk(&mut self) -> std::io::Result<Rxattrwalk> {
        Ok(Rxattrwalk {
            size: self.decode_u64()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_txattrcreate(&mut self) -> std::io::Result<NcTxattrcreate<'b>> {
        Ok(NcTxattrcreate {
            fid: self.decode_u32()?,
            name: self.decode_str()?,
            attr_size: self.decode_u64()?,
            flags: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rxattrcreate(&mut self) -> std::io::Result<Rxattrcreate> {
        Ok(Rxattrcreate {})
    }
    #[allow(clippy::all)]
    fn decode_treaddir(&mut self) -> std::io::Result<Treaddir> {
        Ok(Treaddir {
            fid: self.decode_u32()?,
            offset: self.decode_u64()?,
            count: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_rreaddir(&mut self) -> std::io::Result<NcRreaddir<'b>> {
        Ok(NcRreaddir {
            data: self.decode_nc_direntrydata()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_tfsync(&mut self) -> std::io::Result<Tfsync> {
        Ok(Tfsync {
            fid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rfsync(&mut self) -> std::io::Result<Rfsync> {
        Ok(Rfsync {})
    }
    #[allow(clippy::all)]
    fn decode_nc_tlock(&mut self) -> std::io::Result<NcTlock<'b>> {
        Ok(NcTlock {
            fid: self.decode_u32()?,
            flock: self.decode_nc_flock()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rlock(&mut self) -> std::io::Result<Rlock> {
        Ok(Rlock {
            status: self.decode_lockstatus()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_tgetlock(&mut self) -> std::io::Result<NcTgetlock<'b>> {
        Ok(NcTgetlock {
            fid: self.decode_u32()?,
            flock: self.decode_nc_getlock()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_rgetlock(&mut self) -> std::io::Result<NcRgetlock<'b>> {
        Ok(NcRgetlock {
            flock: self.decode_nc_getlock()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_tlink(&mut self) -> std::io::Result<NcTlink<'b>> {
        Ok(NcTlink {
            dfid: self.decode_u32()?,
            fid: self.decode_u32()?,
            name: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rlink(&mut self) -> std::io::Result<Rlink> {
        Ok(Rlink {})
    }
    #[allow(clippy::all)]
    fn decode_nc_tmkdir(&mut self) -> std::io::Result<NcTmkdir<'b>> {
        Ok(NcTmkdir {
            dfid: self.decode_u32()?,
            name: self.decode_str()?,
            mode: self.decode_u32()?,
            gid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rmkdir(&mut self) -> std::io::Result<Rmkdir> {
        Ok(Rmkdir {
            qid: self.decode_qid()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_trenameat(&mut self) -> std::io::Result<NcTrenameat<'b>> {
        Ok(NcTrenameat {
            olddirfid: self.decode_u32()?,
            oldname: self.decode_str()?,
            newdirfid: self.decode_u32()?,
            newname: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rrenameat(&mut self) -> std::io::Result<Rrenameat> {
        Ok(Rrenameat {})
    }
    #[allow(clippy::all)]
    fn decode_nc_tunlinkat(&mut self) -> std::io::Result<NcTunlinkat<'b>> {
        Ok(NcTunlinkat {
            dirfd: self.decode_u32()?,
            name: self.decode_str()?,
            flags: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_runlinkat(&mut self) -> std::io::Result<Runlinkat> {
        Ok(Runlinkat {})
    }
    #[allow(clippy::all)]
    fn decode_nc_tauth(&mut self) -> std::io::Result<NcTauth<'b>> {
        Ok(NcTauth {
            afid: self.decode_u32()?,
            uname: self.decode_str()?,
            aname: self.decode_str()?,
            n_uname: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rauth(&mut self) -> std::io::Result<Rauth> {
        Ok(Rauth {
            aqid: self.decode_qid()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_tversion(&mut self) -> std::io::Result<NcTversion<'b>> {
        Ok(NcTversion {
            msize: self.decode_u32()?,
            version: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_rversion(&mut self) -> std::io::Result<NcRversion<'b>> {
        Ok(NcRversion {
            msize: self.decode_u32()?,
            version: self.decode_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_tflush(&mut self) -> std::io::Result<Tflush> {
        Ok(Tflush {
            oldtag: self.decode_u16()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rflush(&mut self) -> std::io::Result<Rflush> {
        Ok(Rflush {})
    }
    #[allow(clippy::all)]
    fn decode_nc_twalk(&mut self) -> std::io::Result<NcTwalk<'b>> {
        Ok(NcTwalk {
            fid: self.decode_u32()?,
            newfid: self.decode_u32()?,
            wnames: self.decode_vec_str()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rwalk(&mut self) -> std::io::Result<Rwalk> {
        Ok(Rwalk {
            wqids: self.decode_vec_qid()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_tread(&mut self) -> std::io::Result<Tread> {
        Ok(Tread {
            fid: self.decode_u32()?,
            offset: self.decode_u64()?,
            count: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_rread(&mut self) -> std::io::Result<NcRread<'b>> {
        Ok(NcRread {
            data: self.decode_data_buf()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_nc_twrite(&mut self) -> std::io::Result<NcTwrite<'b>> {
        Ok(NcTwrite {
            fid: self.decode_u32()?,
            offset: self.decode_u64()?,
            data: self.decode_data_buf()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rwrite(&mut self) -> std::io::Result<Rwrite> {
        Ok(Rwrite {
            count: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_tclunk(&mut self) -> std::io::Result<Tclunk> {
        Ok(Tclunk {
            fid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rclunk(&mut self) -> std::io::Result<Rclunk> {
        Ok(Rclunk {})
    }
    #[allow(clippy::all)]
    fn decode_tremove(&mut self) -> std::io::Result<Tremove> {
        Ok(Tremove {
            fid: self.decode_u32()?,
        })
    }
    #[allow(clippy::all)]
    fn decode_rremove(&mut self) -> std::io::Result<Rremove> {
        Ok(Rremove {})
    }
    #[allow(clippy::all)]
    fn decode_nc_msg(&mut self) -> std::io::Result<NcMsg<'b>> {
        let msg_type = MsgType::from_u8(self.decode_u8()?);
        let tag = self.decode_u16()?;
        let body = match msg_type {
            Some(MsgType::Rlerror) => NcFcall::Rlerror(self.decode_rlerror()?),
            Some(MsgType::Tattach) => NcFcall::Tattach(self.decode_nc_tattach()?),
            Some(MsgType::Rattach) => NcFcall::Rattach(self.decode_rattach()?),
            Some(MsgType::Tstatfs) => NcFcall::Tstatfs(self.decode_tstatfs()?),
            Some(MsgType::Rstatfs) => NcFcall::Rstatfs(self.decode_rstatfs()?),
            Some(MsgType::Tlopen) => NcFcall::Tlopen(self.decode_tlopen()?),
            Some(MsgType::Rlopen) => NcFcall::Rlopen(self.decode_rlopen()?),
            Some(MsgType::Tlcreate) => NcFcall::Tlcreate(self.decode_nc_tlcreate()?),
            Some(MsgType::Rlcreate) => NcFcall::Rlcreate(self.decode_rlcreate()?),
            Some(MsgType::Tsymlink) => NcFcall::Tsymlink(self.decode_nc_tsymlink()?),
            Some(MsgType::Rsymlink) => NcFcall::Rsymlink(self.decode_rsymlink()?),
            Some(MsgType::Tmknod) => NcFcall::Tmknod(self.decode_nc_tmknod()?),
            Some(MsgType::Rmknod) => NcFcall::Rmknod(self.decode_rmknod()?),
            Some(MsgType::Trename) => NcFcall::Trename(self.decode_nc_trename()?),
            Some(MsgType::Rrename) => NcFcall::Rrename(self.decode_rrename()?),
            Some(MsgType::Treadlink) => NcFcall::Treadlink(self.decode_treadlink()?),
            Some(MsgType::Rreadlink) => NcFcall::Rreadlink(self.decode_nc_rreadlink()?),
            Some(MsgType::Tgetattr) => NcFcall::Tgetattr(self.decode_tgetattr()?),
            Some(MsgType::Rgetattr) => NcFcall::Rgetattr(self.decode_rgetattr()?),
            Some(MsgType::Tsetattr) => NcFcall::Tsetattr(self.decode_tsetattr()?),
            Some(MsgType::Rsetattr) => NcFcall::Rsetattr(self.decode_rsetattr()?),
            Some(MsgType::Txattrwalk) => NcFcall::Txattrwalk(self.decode_nc_txattrwalk()?),
            Some(MsgType::Rxattrwalk) => NcFcall::Rxattrwalk(self.decode_rxattrwalk()?),
            Some(MsgType::Txattrcreate) => NcFcall::Txattrcreate(self.decode_nc_txattrcreate()?),
            Some(MsgType::Rxattrcreate) => NcFcall::Rxattrcreate(self.decode_rxattrcreate()?),
            Some(MsgType::Treaddir) => NcFcall::Treaddir(self.decode_treaddir()?),
            Some(MsgType::Rreaddir) => NcFcall::Rreaddir(self.decode_nc_rreaddir()?),
            Some(MsgType::Tfsync) => NcFcall::Tfsync(self.decode_tfsync()?),
            Some(MsgType::Rfsync) => NcFcall::Rfsync(self.decode_rfsync()?),
            Some(MsgType::Tlock) => NcFcall::Tlock(self.decode_nc_tlock()?),
            Some(MsgType::Rlock) => NcFcall::Rlock(self.decode_rlock()?),
            Some(MsgType::Tgetlock) => NcFcall::Tgetlock(self.decode_nc_tgetlock()?),
            Some(MsgType::Rgetlock) => NcFcall::Rgetlock(self.decode_nc_rgetlock()?),
            Some(MsgType::Tlink) => NcFcall::Tlink(self.decode_nc_tlink()?),
            Some(MsgType::Rlink) => NcFcall::Rlink(self.decode_rlink()?),
            Some(MsgType::Tmkdir) => NcFcall::Tmkdir(self.decode_nc_tmkdir()?),
            Some(MsgType::Rmkdir) => NcFcall::Rmkdir(self.decode_rmkdir()?),
            Some(MsgType::Trenameat) => NcFcall::Trenameat(self.decode_nc_trenameat()?),
            Some(MsgType::Rrenameat) => NcFcall::Rrenameat(self.decode_rrenameat()?),
            Some(MsgType::Tunlinkat) => NcFcall::Tunlinkat(self.decode_nc_tunlinkat()?),
            Some(MsgType::Runlinkat) => NcFcall::Runlinkat(self.decode_runlinkat()?),
            Some(MsgType::Tauth) => NcFcall::Tauth(self.decode_nc_tauth()?),
            Some(MsgType::Rauth) => NcFcall::Rauth(self.decode_rauth()?),
            Some(MsgType::Tversion) => NcFcall::Tversion(self.decode_nc_tversion()?),
            Some(MsgType::Rversion) => NcFcall::Rversion(self.decode_nc_rversion()?),
            Some(MsgType::Tflush) => NcFcall::Tflush(self.decode_tflush()?),
            Some(MsgType::Rflush) => NcFcall::Rflush(self.decode_rflush()?),
            Some(MsgType::Twalk) => NcFcall::Twalk(self.decode_nc_twalk()?),
            Some(MsgType::Rwalk) => NcFcall::Rwalk(self.decode_rwalk()?),
            Some(MsgType::Tread) => NcFcall::Tread(self.decode_tread()?),
            Some(MsgType::Rread) => NcFcall::Rread(self.decode_nc_rread()?),
            Some(MsgType::Twrite) => NcFcall::Twrite(self.decode_nc_twrite()?),
            Some(MsgType::Rwrite) => NcFcall::Rwrite(self.decode_rwrite()?),
            Some(MsgType::Tclunk) => NcFcall::Tclunk(self.decode_tclunk()?),
            Some(MsgType::Rclunk) => NcFcall::Rclunk(self.decode_rclunk()?),
            Some(MsgType::Tremove) => NcFcall::Tremove(self.decode_tremove()?),
            Some(MsgType::Rremove) => NcFcall::Rremove(self.decode_rremove()?),
            Some(MsgType::Tlerror) | None => return Err(invalid_9p_msg()),
        };
        Ok(NcMsg { tag, body })
    }
}
