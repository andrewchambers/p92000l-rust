use p92000::fcall;
use p92000::fcall::Qid;
use std::borrow::Cow;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::net::TcpListener;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

struct RoExport {}

#[derive(Debug)]
struct RoFid {
    path: PathBuf,
    metadata: fs::Metadata,
    qid: Qid,
    f: Option<std::fs::File>,
}

fn qid_from_metadata(attr: &std::fs::Metadata) -> Qid {
    Qid {
        typ: From::from(attr.file_type()),
        version: 0,
        path: attr.ino(),
    }
}

fn get_dirent_from(p: &Path, offset: u64) -> std::io::Result<fcall::DirEntry<'static>> {
    let metadata = std::fs::metadata(p)?;
    Ok(fcall::DirEntry {
        qid: qid_from_metadata(&metadata),
        offset,
        typ: 0,
        name: Cow::from(p.to_string_lossy().into_owned()),
    })
}

pub fn get_dirent(entry: &fs::DirEntry, offset: u64) -> std::io::Result<fcall::DirEntry<'static>> {
    Ok(fcall::DirEntry {
        qid: qid_from_metadata(&entry.metadata()?),
        offset,
        typ: 0,
        name: Cow::from(entry.file_name().to_string_lossy().into_owned()),
    })
}

// An example filesystem doing many things inefficient but simple ways.
impl p92000::lserver::Filesystem for RoExport {
    type Fid = RoFid;

    fn attach(
        &self,
        _afid: Option<&mut Self::Fid>,
        _uname: &str,
        _aname: &str,
        _n_uname: u32,
    ) -> Result<(RoFid, fcall::Rattach), fcall::Rlerror> {
        let path = std::env::current_dir()?;
        let metadata = fs::symlink_metadata(&path)?;
        let qid = qid_from_metadata(&metadata);

        Ok((
            RoFid {
                path,
                qid,
                metadata,
                f: None,
            },
            fcall::Rattach { qid },
        ))
    }

    fn getattr(
        &self,
        fid: &mut Self::Fid,
        req_mask: fcall::GetattrMask,
    ) -> Result<fcall::Rgetattr, fcall::Rlerror> {
        Ok(fcall::Rgetattr {
            valid: req_mask,
            stat: (&fid.metadata).into(),
            qid: fid.qid,
        })
    }

    fn walk(
        &self,
        fid: &mut Self::Fid,
        wnames: &[Cow<'_, str>],
    ) -> Result<(Option<Self::Fid>, fcall::Rwalk), fcall::Rlerror> {
        let mut wqids = Vec::new();
        let mut path = fid.path.clone();

        for (_i, name) in wnames.iter().enumerate() {
            path.push(name.as_ref());
            if let Ok(metadata) = fs::symlink_metadata(&path) {
                let qid = qid_from_metadata(&metadata);
                wqids.push(qid);
            } else {
                return Ok((None, fcall::Rwalk { wqids }));
            }
        }

        let metadata = fs::symlink_metadata(&path)?;
        let qid = qid_from_metadata(&metadata);

        Ok((
            Some(RoFid {
                path,
                qid,
                metadata,
                f: None,
            }),
            fcall::Rwalk { wqids },
        ))
    }

    fn lopen(
        &self,
        fid: &mut Self::Fid,
        _flags: fcall::LOpenFlags,
    ) -> Result<fcall::Rlopen, fcall::Rlerror> {
        Ok(fcall::Rlopen {
            qid: fid.qid,
            iounit: 0,
        })
    }

    fn readdir(
        &self,
        fid: &mut Self::Fid,
        off: u64,
        count: u32,
    ) -> Result<fcall::Rreaddir<'static>, fcall::Rlerror> {
        let mut dirents = fcall::DirEntryData::<'static>::new();

        let offset = if off == 0 {
            dirents.push(get_dirent_from(&PathBuf::from("."), 0)?);
            dirents.push(get_dirent_from(&PathBuf::from(".."), 1)?);
            off
        } else {
            off - 1
        } as usize;

        // Note, In a 'production' filesystem you should try to preserve state between calls.
        let entries = std::fs::read_dir(&fid.path)?.skip(offset);

        let mut i = offset;
        for entry in entries {
            let entry = entry?;
            let dirent = get_dirent(&entry, 2 + i as u64)?;
            if dirents.size() + dirent.size() > count {
                break;
            }
            dirents.push(dirent);
            i += 1;
        }

        Ok(fcall::Rreaddir { data: dirents })
    }

    fn read(
        &self,
        fid: &mut Self::Fid,
        offset: u64,
        buf: &mut [u8],
    ) -> Result<usize, fcall::Rlerror> {
        if fid.f.is_none() {
            fid.f = Some(std::fs::File::open(&fid.path)?);
        }
        match fid.f {
            Some(ref mut f) => {
                f.seek(std::io::SeekFrom::Start(offset))?;
                let n = f.read(buf)?;
                Ok(n)
            }
            _ => unreachable!(),
        }
    }
}

fn main() {
    let addr = "127.0.0.1:7878";
    let listener = TcpListener::bind(addr).unwrap();

    let mut fs = RoExport {};

    println!("listening on {}", addr);
    for stream in listener.incoming() {
        let mut stream1 = stream.unwrap();
        let mut stream2 = stream1.try_clone().unwrap();
        p92000::lserver::serve_single_threaded(&mut stream1, &mut stream2, &mut fs);
    }
}
