(import sh)

(def msgs ~[Rlerror [ecode u32]
            Tattach [fid u32
                     afid u32
                     uname String
                     aname String
                     n_uname u32]
            Rattach [qid Qid]
            Tstatfs [fid u32]
            Rstatfs [statfs Statfs]
            Tlopen [fid u32
                    flags u32]
            Rlopen [qid Qid
                    iounit u32]
            Tlcreate [fid u32
                      name String
                      flags u32
                      mode u32
                      gid u32]
            Rlcreate [qid Qid
                      iounit u32]
            Tsymlink [fid u32
                      name String
                      symtgt String
                      gid u32]
            Rsymlink [qid Qid]
            Tmknod [dfid u32
                    name String
                    mode u32
                    major u32
                    minor u32
                    gid u32]
            Rmknod [qid Qid]
            Trename [fid u32
                     dfid u32
                     name String]
            Rrename []
            Treadlink [fid u32]
            Rreadlink [target String]
            Tgetattr [fid u32
                      req_mask GetattrMask]
            Rgetattr [valid GetattrMask
                      qid Qid
                      stat Stat]
            Tsetattr [fid u32
                      valid SetattrMask
                      stat SetAttr]
            Rsetattr []
            Txattrwalk [fid u32
                        newfid u32
                        name String]
            Rxattrwalk [size u64]
            Txattrcreate [fid u32
                          name String
                          attr_size u64
                          flags u32]
            Rxattrcreate []
            Treaddir [fid u32
                      offset u64
                      count u32]
            Rreaddir [data DirEntryData]
            Tfsync [fid u32]
            Rfsync []
            Tlock [fid u32
                   flock Flock]
            Rlock [status LockStatus]
            Tgetlock [fid u32
                      flock Getlock]
            Rgetlock [flock Getlock]
            Tlink [dfid u32
                   fid u32
                   name String]
            Rlink []
            Tmkdir [dfid u32
                    name String
                    mode u32
                    gid u32]
            Rmkdir [qid Qid]
            Trenameat [olddirfid u32
                       oldname String
                       newdirfid u32
                       newname String]
            Rrenameat []
            Tunlinkat [dirfd u32
                       name String
                       flags u32]
            Runlinkat []
            Tauth [afid u32
                   uname String
                   aname String
                   n_uname u32]
            Rauth [aqid Qid]
            Tversion [msize u32
                      version String]
            Rversion [msize u32
                      version String]
            Tflush [oldtag u16]
            Rflush []
            Twalk [fid u32
                   newfid u32
                   wnames Vec_String]
            Rwalk [wqids Vec_Qid]
            Tread [fid u32
                   offset u64
                   count u32]
            Rread [data Data]
            Twrite [fid u32
                    offset u64
                    data Data]
            Rwrite [count u32]
            Tclunk [fid u32]
            Rclunk []
            Tremove [fid u32]
            Rremove []])


(def supplemental-types ~[Qid [typ QidType
                               version u32
                               path u64]
                          Statfs [typ u32
                                  bsize u32
                                  blocks u64
                                  bfree u64
                                  bavail u64
                                  files u64
                                  ffree u64
                                  fsid u64
                                  namelen u32]
                          Time [sec u64
                                nsec u64]
                          Stat [mode u32
                                uid u32
                                gid u32
                                nlink u64
                                rdev u64
                                size u64
                                blksize u64
                                blocks u64
                                atime Time
                                mtime Time
                                ctime Time]
                          SetAttr [mode u32
                                   uid u32
                                   gid u32
                                   size u64
                                   atime Time
                                   mtime Time]
                          DirEntry [qid Qid
                                    offset u64
                                    typ u8
                                    name String]
                          Flock [typ LockType
                                 flags LockFlag
                                 start u64
                                 length u64
                                 proc_id u32
                                 client_id String]
                          Getlock [typ LockType
                                   start u64
                                   length u64
                                   proc_id u32
                                   client_id String]])


(def enc-dec-types
  [;supplemental-types ;msgs])

(defn rust-type-name
  [tname]
  (def tname (string tname))
  (case tname
    "Vec_Qid" "Vec<Qid>"
    "Vec_String" "Vec<String>"
    "Data" "Vec<u8>"
    tname))

(defn no-copy-rust-type-name
  [tname]
  (def tname (string tname))
  (cond
    (= tname "String") "&'b str"
    (= tname "Vec_String") "Vec<&'b str>"
    (= tname "Data") "&'b [u8]"
    (or
      (= "Tattach" tname)
      (= "Tlcreate" tname)
      (= "Tsymlink" tname)
      (= "Tmknod" tname)
      (= "Trename" tname)
      (= "Rreadlink" tname)
      (= "Txattrwalk" tname)
      (= "Txattrcreate" tname)
      (= "Rreaddir" tname)
      (= "Tlock" tname)
      (= "Tgetlock" tname)
      (= "Rgetlock" tname)
      (= "Tlink" tname)
      (= "Tmkdir" tname)
      (= "Trenameat" tname)
      (= "Tunlinkat" tname)
      (= "Tauth" tname)
      (= "Tversion" tname)
      (= "Rversion" tname)
      (= "Twalk" tname)
      (= "Rread" tname)
      (= "Twrite" tname)
      (= "DirEntry" tname)
      (= "DirEntryData" tname)
      (= "Flock" tname)
      (= "Getlock" tname)) (string "Nc" tname "<'b>")
    tname))

(defn type-has-no-copy-variant?
  [tname]
  (not= (string tname) (no-copy-rust-type-name tname)))

(defn fn-name-normalize-type
  [name]
  (string/ascii-lower name))

(defn no-copy-fn-name-normalize-type
  [name]
  (def name (string name))
  (if (type-has-no-copy-variant? name)
    (case name
      "String" "str"
      "Vec_String" "vec_str"
      "Data" "data_buf"
      (string "nc_" (fn-name-normalize-type name)))
    (fn-name-normalize-type name)))

(def decoder-impl-prelude
  `
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
        if self.buf.len() < n {
            Err(invalid_9p_msg())
        } else {
            match std::str::from_utf8(&self.buf[..n]) {
                Ok(s) => {
                    self.buf = &self.buf[n..];
                    Ok(s)
                }
                Err(_) => Err(invalid_9p_msg()),
            }
        }
    }

    fn decode_data_buf(&mut self) -> std::io::Result<&'b [u8]> {
        let n = self.decode_u32()? as usize;
        if self.buf.len() < n {
            Err(invalid_9p_msg())
        } else {
            let v = &self.buf[..n];
            self.buf = &self.buf[n..];
            Ok(v)
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

`)

(defn print-types
  [types]
  (each [name fields] (partition 2 types)
    (when (type-has-no-copy-variant? name)
      (print "#[derive(Clone, Debug)]")
      (print "pub struct " (no-copy-rust-type-name name) " {")
      (each [fname ftype] (partition 2 fields)
        (print "  pub " fname ": " (no-copy-rust-type-name ftype) ","))
      (print "}"))
    (print "#[derive(Clone, Debug)]")
    (print "pub struct " name " {")
    (each [fname ftype] (partition 2 fields)
      (print "  pub " fname ": " (rust-type-name ftype) ","))
    (print "}")))

(defn print-msg-types
  []
  (print-types msgs)
  (each [name fields] (partition 2 msgs)
    (print "impl From<" name "> for Fcall {")
    (print "  fn from(v: " name ") -> Fcall {")
    (print "    Fcall::" name "(v)")
    (print "  }")
    (print "}")))

(defn print-supplemental-types
  []
  (print-types supplemental-types))

(defn print-fcall-types
  []
  (print "#[derive(Clone, Debug)]")
  (print "pub enum Fcall {")
  (each [name _] (partition 2 msgs)
    (print "  pub " name "(" name "),"))
  (print "}")
  (print "#[derive(Clone, Debug)]")
  (print "pub enum NcFcall<'b> {")
  (each [name _] (partition 2 msgs)
    (print "  pub " name "(" (no-copy-rust-type-name name) "),"))
  (print "}"))

(defn print-msg-type-from-fcall
  []
  (print "impl<'a> From<&'a Fcall> for MsgType {")
  (print "  fn from(fcall: &'a Fcall) -> MsgType {")
  (print "    match *fcall {")
  (each [name _] (partition 2 msgs)
    (print "      Fcall::" name "(_) => MsgType::" name ","))
  (print "    }")
  (print "  }")
  (print "}")
  (print "impl<'a, 'b> From<&'a NcFcall<'b>> for MsgType {")
  (print "  fn from(fcall: &'a NcFcall<'b>) -> MsgType {")
  (print "    match *fcall {")
  (each [name _] (partition 2 msgs)
    (print "      NcFcall::" name "(_) => MsgType::" name ","))
  (print "    }")
  (print "  }")
  (print "}"))

(defn print-decoder-impl
  []
  (print decoder-impl-prelude)
  (each [name fields] (partition 2 enc-dec-types)
    (print "#[allow(clippy::all)]")
    (print "fn decode_" (no-copy-fn-name-normalize-type name) " (&mut self) -> std::io::Result<" (no-copy-rust-type-name name) "> {")
    (print "  Ok(" (string (when (type-has-no-copy-variant? name) "Nc") name) "{")
    (each [fname ftype] (partition 2 fields)
      (print "    " fname ": self.decode_" (no-copy-fn-name-normalize-type ftype) "()?,"))
    (print "  })")
    (print "}"))
  (print "#[allow(clippy::all)]")
  (print "  fn decode_nc_msg(&mut self) -> std::io::Result<NcMsg<'b>> {")
  (print "    let msg_type = MsgType::from_u8(self.decode_u8()?);")
  (print "    let tag = self.decode_u16()?;")
  (print "    let body = match msg_type {")
  (each [name _] (partition 2 msgs)
    (print "    Some(MsgType::" name ") => NcFcall::" name "(self.decode_" (no-copy-fn-name-normalize-type name) "()?),"))
  (print "    Some(MsgType::Tlerror) | None => return Err(invalid_9p_msg()),")
  (print "  };")
  (print "  Ok(NcMsg {tag, body})")
  (print "}")
  (print "}"))

(defn print-encoder-impl
  []
  (each [name fields] (partition 2 enc-dec-types)
    (def vpfx (if (empty? fields) "_" ""))
    (when (type-has-no-copy-variant? name)
      (print "#[allow(clippy::all)]")
      (print "fn encode_" (no-copy-fn-name-normalize-type name) "<'a, 'b, W: Write>("vpfx"w: &'a mut W, " vpfx "v: &"(no-copy-rust-type-name name)") -> std::io::Result<()> {")
      (each [fname ftype] (partition 2 fields)
        (print "  encode_" (no-copy-fn-name-normalize-type ftype) "(w, &v." fname ")?;"))
      (print "  Ok(())")
      (print "}"))
    (print "#[allow(clippy::all)]")
    (print "fn encode_" (fn-name-normalize-type name) "<W: Write>("vpfx"w: &mut W, " vpfx "v: &"(rust-type-name name)") -> std::io::Result<()> {")
    (each [fname ftype] (partition 2 fields)
      (print "  encode_" (fn-name-normalize-type ftype) "(w, &v." fname ")?;"))
    (print "  Ok(())")
    (print "}"))

  (print "pub fn encode_nc_msg<W: Write>(w: &mut W, v: &NcMsg) -> std::io::Result<()> {")
  (print "  let typ = MsgType::from(&v.body);")
  (print "  encode_u8(w, &(typ as u8))?;")
  (print "  encode_u16(w, &v.tag)?;")
  (print "  match v.body {")
  (each [name _] (partition 2 msgs)
    (print "    NcFcall::" name "(ref v) => encode_" (no-copy-fn-name-normalize-type name) "(w, v)?,"))
  (print "  };")
  (print "  Ok(())")
  (print "}")
  (print "pub fn encode_msg<W: Write>(w: &mut W, v: &Msg) -> std::io::Result<()> {")
  (print "  let typ = MsgType::from(&v.body);")
  (print "  encode_u8(w, &(typ as u8))?;")
  (print "  encode_u16(w, &v.tag)?;")
  (print "  match v.body {")
  (each [name _] (partition 2 msgs)
    (print "    Fcall::" name "(ref v) => encode_" (fn-name-normalize-type name) "(w, v)?,"))
  (print "  };")
  (print "  Ok(())")
  (print "}"))

(defn print-convert-impl
  []
  (each [name fields] (partition 2 enc-dec-types)
    (when (type-has-no-copy-variant? name)
      (print "impl<'b> From<&" (no-copy-rust-type-name name) "> for " (rust-type-name name) "{")
      (print "  #[allow(clippy::all)]")
      (print "  fn from(v: &" (no-copy-rust-type-name name)") -> " (rust-type-name name) " {")
      (print "    " (rust-type-name name) "{")
      (each [fname ftype] (partition 2 fields)
        (if (type-has-no-copy-variant? ftype)
          (if (= ftype 'Vec_String)
            (print "      " fname ": v." fname ".iter().map(|x| x.to_string()).collect(),")
            (print "      " fname ": v." fname ".clone().into(),"))

          (print "      " fname ": v." fname ".clone(),")))
      (print "    }")
      (print "  }")
      (print "}")

      (print "impl<'b> From<" (no-copy-rust-type-name name) "> for " (rust-type-name name) "{")
      (print "  #[allow(clippy::all)]")
      (print "  fn from(v: " (no-copy-rust-type-name name)") -> " (rust-type-name name) " {")
      (print "    " (rust-type-name name) "{")
      (each [fname ftype] (partition 2 fields)
        (if (type-has-no-copy-variant? ftype)
          (if (= ftype 'Vec_String)
            (print "      " fname ": v." fname ".iter().map(|x| x.to_string()).collect(),")
            (print "      " fname ": v." fname ".into(),"))

          (print "      " fname ": v." fname ",")))
      (print "    }")
      (print "  }")
      (print "}")))

  (print "impl<'a, 'b> From<&'a NcFcall<'b>> for Fcall {")
  (print "  fn from(fcall: &'a NcFcall<'b>) -> Fcall {")
  (print "    match fcall {")
  (each [name _] (partition 2 msgs)
    (if (type-has-no-copy-variant? name)
      (print "      NcFcall::" name "(ref v) => Fcall::" name "(v.into()),")
      (print "      NcFcall::" name "(ref v) => Fcall::" name "(v.clone()),")))
  (print "    }")
  (print "  }")
  (print "}"))

(defn write-src/fcall_types.gen.rs
  []
  (with-dyns [:out @""]
    (print-supplemental-types)
    (print-msg-types)
    (print-fcall-types)
    (print-msg-type-from-fcall)
    (spit "src/fcall_types.gen.rs" (dyn :out))
    (sh/$ rustfmt "src/fcall_types.gen.rs")))

(defn write-src/decoder_impl.gen.rs
  []
  (with-dyns [:out @""]
    (print-decoder-impl)
    (spit "src/decoder_impl.gen.rs" (dyn :out))
    (sh/$ rustfmt "src/decoder_impl.gen.rs")))

(defn write-src/encoder_impl.gen.rs
  []
  (with-dyns [:out @""]
    (print-encoder-impl)
    (spit "src/encoder_impl.gen.rs" (dyn :out))
    (sh/$ rustfmt "src/encoder_impl.gen.rs")))

(defn write-src/convert_impl.gen.rs
  []
  (with-dyns [:out @""]
    (print-convert-impl)
    (spit "src/convert_impl.gen.rs" (dyn :out))
    (sh/$ rustfmt "src/convert_impl.gen.rs")))

#(print-msg-types)
#(print-supplemental-types)
#(print-decoder-impl)
#(print-fcall-types)
#(print-msg-type-from-fcall)
#(print-encoder-impl)
#(print-convert-impl)
(write-src/fcall_types.gen.rs)
(write-src/decoder_impl.gen.rs)
(write-src/encoder_impl.gen.rs)
(write-src/convert_impl.gen.rs)
