use super::fcall;
use std::io::{Read, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::time::Duration;

pub trait Transport: Read + Write + Send + Sync {
    fn set_read_timeout(&mut self, dur: Option<Duration>) -> Result<(), std::io::Error>;

    fn read_timeout(&self) -> Result<Option<Duration>, std::io::Error>;

    fn shutdown(&self) -> Result<(), std::io::Error>;
}

impl Transport for TcpStream {
    fn set_read_timeout(&mut self, d: Option<Duration>) -> Result<(), std::io::Error> {
        TcpStream::set_read_timeout(self, d)
    }

    fn read_timeout(&self) -> Result<Option<Duration>, std::io::Error> {
        TcpStream::read_timeout(self)
    }

    fn shutdown(&self) -> Result<(), std::io::Error> {
        TcpStream::shutdown(self, std::net::Shutdown::Both)
    }
}

#[cfg(unix)]
impl Transport for UnixStream {
    fn set_read_timeout(&mut self, d: Option<Duration>) -> Result<(), std::io::Error> {
        UnixStream::set_read_timeout(self, d)
    }

    fn read_timeout(&self) -> Result<Option<Duration>, std::io::Error> {
        UnixStream::read_timeout(self)
    }

    fn shutdown(&self) -> Result<(), std::io::Error> {
        UnixStream::shutdown(self, std::net::Shutdown::Both)
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
pub fn read_to_buf_timeout<T: Transport>(
    conn: &mut T,
    fcall_buf: &mut Vec<u8>,
    timeout: Duration,
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
) -> Result<fcall::TaggedFcall<'a>, std::io::Error> {
    read_to_buf(r, buf)?;
    fcall::TaggedFcall::decode(&buf[..])
}

fn write_u8<W: Write>(w: &mut W, v: u8) -> std::io::Result<()> {
    w.write_all(&[v])?;
    Ok(())
}

fn write_u16<W: Write>(w: &mut W, v: u16) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes()[..])?;
    Ok(())
}

fn write_u32<W: Write>(w: &mut W, v: u32) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes()[..])?;
    Ok(())
}

fn write_u64<W: Write>(w: &mut W, v: u64) -> std::io::Result<()> {
    w.write_all(&v.to_le_bytes()[..])?;
    Ok(())
}

pub fn write<W: Write>(
    w: &mut W,
    buf: &mut Vec<u8>,
    fcall: &fcall::TaggedFcall,
) -> std::io::Result<()> {
    buf.truncate(0);
    match fcall {
        fcall::TaggedFcall {
            tag,
            fcall: fcall::Fcall::Rread(fcall::Rread { data }),
        } => {
            // Zero copy Rread path.
            let sz = 4 + 1 + 2 + 4 + data.len();
            if sz > buf.capacity() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "9p message overflows msize",
                ));
            }
            let mut cursor = std::io::Cursor::new(buf);
            write_u32(&mut cursor, sz as u32)?;
            write_u8(&mut cursor, 117)?;
            write_u16(&mut cursor, *tag)?;
            write_u32(&mut cursor, data.len() as u32)?;
            let buf = cursor.into_inner();
            // XXX: Could be a vectored write all if it were stable.
            w.write_all(&buf[..])?;
            w.write_all(&data[..])?;
            Ok(())
        }
        fcall::TaggedFcall {
            tag,
            fcall: fcall::Fcall::Twrite(fcall::Twrite { fid, offset, data }),
        } => {
            // Zero copy Twrite path.
            let sz = 4 + 1 + 2 + 4 + 8 + 4 + data.len();
            if sz > buf.capacity() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "9p message overflows msize",
                ));
            }
            let mut cursor = std::io::Cursor::new(buf);
            write_u32(&mut cursor, sz as u32)?;
            write_u8(&mut cursor, 118)?;
            write_u16(&mut cursor, *tag)?;
            write_u32(&mut cursor, *fid)?;
            write_u64(&mut cursor, *offset)?;
            write_u32(&mut cursor, data.len() as u32)?;
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
