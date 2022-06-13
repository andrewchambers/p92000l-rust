/*
Copyright (c) 2019 Yoran Heling
Permission is hereby granted, free of charge, to any person obtaining
a copy of this software and associated documentation files (the
"Software"), to deal in the Software without restriction, including
without limitation the rights to use, copy, modify, merge, publish,
distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to
the following conditions:
The above copyright notice and this permission notice shall be included
in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use super::transport::Transport;
use std::fmt;
use std::io;
use std::net;
use std::net::ToSocketAddrs;
#[cfg(unix)]
use std::os::unix::net as unix;
#[cfg(unix)]
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

/// Wrapper for a `std::net::SocketAddr` or UNIX socket path.
///
/// UNIX sockets are prefixed with 'unix:' when parsing and formatting.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SocketAddr {
    Inet(net::SocketAddr),
    #[cfg(unix)]
    Unix(PathBuf),
}

impl From<net::SocketAddr> for SocketAddr {
    fn from(s: net::SocketAddr) -> SocketAddr {
        SocketAddr::Inet(s)
    }
}

#[cfg(unix)]
impl From<unix::SocketAddr> for SocketAddr {
    fn from(s: unix::SocketAddr) -> SocketAddr {
        SocketAddr::Unix(match s.as_pathname() {
            None => Path::new("unnamed").to_path_buf(),
            Some(p) => p.to_path_buf(),
        })
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SocketAddr::Inet(n) => write!(f, "{}", n),
            #[cfg(unix)]
            SocketAddr::Unix(n) => write!(f, "unix:{}", n.to_string_lossy()),
        }
    }
}

impl FromStr for SocketAddr {
    type Err = net::AddrParseError;

    #[cfg(unix)]
    fn from_str(s: &str) -> Result<SocketAddr, net::AddrParseError> {
        if s.starts_with("unix:") {
            Ok(SocketAddr::Unix(
                Path::new(s.trim_start_matches("unix:")).to_path_buf(),
            ))
        } else {
            s.parse().map(SocketAddr::Inet)
        }
    }

    #[cfg(not(unix))]
    fn from_str(s: &str) -> Result<SocketAddr, net::AddrParseError> {
        s.parse().map(SocketAddr::Inet)
    }
}

pub enum ResolveAddressError {
    ParseError(std::net::AddrParseError),
    IoError(std::io::Error),
}

impl std::fmt::Display for ResolveAddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ResolveAddressError::ParseError(err) => err.fmt(f),
            ResolveAddressError::IoError(err) => err.fmt(f),
        }
    }
}

impl SocketAddr {
    pub fn resolve(s: &str) -> Result<SocketAddr, ResolveAddressError> {
        if s.starts_with("unix:") {
            match s.parse() {
                Ok(addr) => Ok(addr),
                Err(err) => Err(ResolveAddressError::ParseError(err)),
            }
        } else {
            match s.to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(addr) = addrs.next() {
                        Ok(addr.into())
                    } else {
                        Err(ResolveAddressError::IoError(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "failed to resolve an address",
                        )))
                    }
                }
                Err(err) => Err(ResolveAddressError::IoError(err)),
            }
        }
    }

    pub fn is_unix(&self) -> bool {
        match self {
            #[cfg(unix)]
            SocketAddr::Unix(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum Socket {
    Inet(net::TcpStream),
    #[cfg(unix)]
    Unix(unix::UnixStream),
}

impl From<net::TcpStream> for Socket {
    fn from(s: net::TcpStream) -> Socket {
        Socket::Inet(s)
    }
}

#[cfg(unix)]
impl From<unix::UnixStream> for Socket {
    fn from(s: unix::UnixStream) -> Socket {
        Socket::Unix(s)
    }
}

impl Socket {
    pub fn connect(s: &SocketAddr) -> io::Result<Socket> {
        match s {
            SocketAddr::Inet(s) => net::TcpStream::connect(s).map(Socket::Inet),
            #[cfg(unix)]
            SocketAddr::Unix(s) => unix::UnixStream::connect(s).map(Socket::Unix),
        }
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        match self {
            Socket::Inet(s) => s.local_addr().map(SocketAddr::Inet),
            #[cfg(unix)]
            Socket::Unix(s) => s.local_addr().map(|e| e.into()),
        }
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match self {
            Socket::Inet(s) => s.peer_addr().map(SocketAddr::Inet),
            #[cfg(unix)]
            Socket::Unix(s) => s.peer_addr().map(|e| e.into()),
        }
    }

    pub fn try_clone(&self) -> io::Result<Socket> {
        match self {
            Socket::Inet(s) => Ok(Socket::Inet(net::TcpStream::try_clone(s)?)),
            #[cfg(unix)]
            Socket::Unix(s) => Ok(Socket::Unix(unix::UnixStream::try_clone(s)?)),
        }
    }
}

impl io::Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Socket::Inet(s) => s.read(buf),
            #[cfg(unix)]
            Socket::Unix(s) => s.read(buf),
        }
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut]) -> io::Result<usize> {
        match self {
            Socket::Inet(s) => s.read_vectored(bufs),
            #[cfg(unix)]
            Socket::Unix(s) => s.read_vectored(bufs),
        }
    }
}

impl io::Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Socket::Inet(s) => s.write(buf),
            #[cfg(unix)]
            Socket::Unix(s) => s.write(buf),
        }
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice]) -> io::Result<usize> {
        match self {
            Socket::Inet(s) => s.write_vectored(bufs),
            #[cfg(unix)]
            Socket::Unix(s) => s.write_vectored(bufs),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Socket::Inet(s) => s.flush(),
            #[cfg(unix)]
            Socket::Unix(s) => s.flush(),
        }
    }
}

impl Transport for Socket {
    fn set_read_timeout(&mut self, t: Option<Duration>) -> io::Result<()> {
        match self {
            Socket::Inet(s) => s.set_read_timeout(t),
            #[cfg(unix)]
            Socket::Unix(s) => s.set_read_timeout(t),
        }
    }

    fn read_timeout(&self) -> io::Result<Option<Duration>> {
        match self {
            Socket::Inet(s) => s.read_timeout(),
            #[cfg(unix)]
            Socket::Unix(s) => s.read_timeout(),
        }
    }

    fn shutdown(&self) -> io::Result<()> {
        match self {
            Socket::Inet(s) => s.shutdown(std::net::Shutdown::Both),
            #[cfg(unix)]
            Socket::Unix(s) => s.shutdown(std::net::Shutdown::Both),
        }
    }
}

#[derive(Debug)]
pub enum SocketListener {
    Inet(net::TcpListener),
    #[cfg(unix)]
    Unix(unix::UnixListener),
}

impl From<net::TcpListener> for SocketListener {
    fn from(s: net::TcpListener) -> SocketListener {
        SocketListener::Inet(s)
    }
}

#[cfg(unix)]
impl From<unix::UnixListener> for SocketListener {
    fn from(s: unix::UnixListener) -> SocketListener {
        SocketListener::Unix(s)
    }
}

impl SocketListener {
    pub fn bind(s: &SocketAddr) -> io::Result<SocketListener> {
        match s {
            SocketAddr::Inet(s) => net::TcpListener::bind(s).map(SocketListener::Inet),
            #[cfg(unix)]
            SocketAddr::Unix(s) => unix::UnixListener::bind(s).map(SocketListener::Unix),
        }
    }

    /// Same as `bind()`, but for UNIX sockets this will try to re-bind to the path if the process
    /// that used to listen to this address is no longer running. It can also optionally set the
    /// permissions of the UNIX socket.
    ///
    /// # Limitations
    ///
    /// Trying to bind to the same UNIX socket path from multiple processes is subject to a race
    /// condition.
    ///
    /// The permissions are set *after* performing the `bind()` operation, so if the default umask
    /// is less restrictive than the given mode, there is a short window where an unprivileged
    /// process could attempt to connect to the socket.
    pub fn bind_reuse(s: &SocketAddr, _mode: Option<u32>) -> io::Result<SocketListener> {
        let b = match (Self::bind(s), s) {
            #[cfg(unix)]
            (Err(ref e), &SocketAddr::Unix(ref p)) if e.kind() == io::ErrorKind::AddrInUse => {
                let e = io::Error::last_os_error();

                // Make sure it is a socket in the first place (we don't want to overwrite a
                // regular file)
                use std::os::unix::fs::FileTypeExt;
                match std::fs::symlink_metadata(p) {
                    Ok(ref m) if m.file_type().is_socket() => (),
                    _ => return Err(e),
                };

                // Try to connect to the socket to see if it's still alive.
                match Socket::connect(s) {
                    // Not alive, delete the socket and try to bind again.
                    Err(ref e2) if e2.kind() == io::ErrorKind::ConnectionRefused => {
                        std::fs::remove_file(p).and_then(|_| Self::bind(s))?
                    }
                    _ => return Err(e),
                }
            }
            (Err(e), _) => return Err(e),
            (Ok(l), _) => l,
        };

        #[cfg(unix)]
        #[allow(clippy::single_match)]
        match (_mode, s) {
            (Some(perm), &SocketAddr::Unix(ref p)) => {
                use std::fs::{set_permissions, Permissions};
                use std::os::unix::fs::PermissionsExt;
                set_permissions(p, Permissions::from_mode(perm))?;
            }
            _ => (),
        }
        Ok(b)
    }

    pub fn accept(&self) -> io::Result<(Socket, SocketAddr)> {
        match self {
            SocketListener::Inet(l) => l.accept().map(|(s, e)| (s.into(), e.into())),
            #[cfg(unix)]
            SocketListener::Unix(l) => l.accept().map(|(s, e)| (s.into(), e.into())),
        }
    }
}

#[test]
fn test_socketaddr_inet() {
    let ip4 = "127.0.0.1:10".parse::<net::SocketAddr>().unwrap();
    let ip6 = "[::20]:10".parse::<net::SocketAddr>().unwrap();
    assert_eq!(
        ip4.to_string(),
        "127.0.0.1:10".parse::<SocketAddr>().unwrap().to_string()
    );
    assert_eq!(ip4.to_string(), SocketAddr::from(ip4).to_string());
    assert_eq!(
        ip6.to_string(),
        "[::20]:10".parse::<SocketAddr>().unwrap().to_string()
    );
    assert_eq!(ip6.to_string(), SocketAddr::from(ip6).to_string());
}

#[test]
#[cfg(unix)]
fn test_socketaddr_unix() {
    assert_eq!(
        "unix:/tmp/sock".parse::<SocketAddr>().unwrap().to_string(),
        "unix:/tmp/sock"
    );
    assert!("/tmp/sock".parse::<SocketAddr>().is_err());
}
