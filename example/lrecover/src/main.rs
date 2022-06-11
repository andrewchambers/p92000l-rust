use log::{debug, error, info};
use p92000::fcall;
use p92000::fcall::{Fcall, FcallType};
use p92000::lerrno;
use std::collections::HashMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::ops::Add;
use std::sync::Arc;
use std::sync::Mutex;
use std::time;

struct ProxyState {
    server_addr: String,
    client_conn: TcpStream,
    server_conn: TcpStream,
    rversion: fcall::Rversion<'static>,
    server_state: Arc<Mutex<ShadowServerState>>,
}

#[derive(Debug)]
enum AttachChange {
    AddOnSuccess((u32, fcall::Tattach<'static>)),
    Remove(u32),
    None,
}

#[derive(Debug)]
struct ShadowServerState {
    // mapping of fids to Tattach so we can restablish them on reconnect.
    attach_fids: HashMap<u32, fcall::Tattach<'static>>,
    inflight_tags: HashMap<u16, AttachChange>,
}

impl ShadowServerState {
    fn new() -> ShadowServerState {
        ShadowServerState {
            attach_fids: HashMap::new(),
            inflight_tags: HashMap::new(),
        }
    }

    fn on_fcall(&mut self, buf: &[u8]) {
        let fcall_type = match FcallType::from_u8(buf[4]) {
            Some(t) => t,
            None => return,
        };
        // For speed, we often don't even both decoding the full buffer.
        match fcall_type {
            FcallType::Tattach | FcallType::Tclunk | FcallType::Tremove => {
                if let Ok(tagged_fcall) = fcall::TaggedFcall::decode(buf) {
                    match &tagged_fcall.fcall {
                        Fcall::Tattach(tattach) => {
                            self.inflight_tags.insert(
                                tagged_fcall.tag,
                                AttachChange::AddOnSuccess(
                                    (tattach.fid, tattach.clone_static())
                                ),
                            );
                        }
                        Fcall::Tclunk(fcall::Tclunk { fid }) => {
                            self
                            .inflight_tags
                            .insert(tagged_fcall.tag, AttachChange::Remove(*fid));
                        }
                        Fcall::Tremove(fcall::Tremove { fid }) => {
                            self
                            .inflight_tags
                            .insert(tagged_fcall.tag, AttachChange::Remove(*fid));
                        }
                        _ => (),
                    }
                }
            }
            FcallType::Twalk // For now we simply ignore clones of attach points.
            | FcallType::Tstatfs
            | FcallType::Tlopen
            | FcallType::Tlcreate
            | FcallType::Tsymlink
            | FcallType::Tmknod
            | FcallType::Treadlink
            | FcallType::Tgetattr
            | FcallType::Tsetattr
            | FcallType::Treaddir
            | FcallType::Tfsync
            | FcallType::Tmkdir
            | FcallType::Tflush
            | FcallType::Tread
            | FcallType::Twrite
            | FcallType::Trename
            | FcallType::Tlink
            | FcallType::Trenameat
            | FcallType::Tunlinkat
            | FcallType::Tlock
            | FcallType::Tgetlock
            | FcallType::Tauth
            | FcallType::Txattrwalk
            | FcallType::Txattrcreate
            | FcallType::Tversion => {
                let tag = u16::from_le_bytes(buf[5..7].try_into().unwrap());
                self
                .inflight_tags
                .insert(tag, AttachChange::None);
            }
             FcallType::Rwalk
            | FcallType::Rstatfs
            | FcallType::Rlopen
            | FcallType::Rlcreate
            | FcallType::Rsymlink
            | FcallType::Rmknod
            | FcallType::Rreadlink
            | FcallType::Rgetattr
            | FcallType::Rsetattr
            | FcallType::Rreaddir
            | FcallType::Rfsync
            | FcallType::Rmkdir
            | FcallType::Rflush
            | FcallType::Rread
            | FcallType::Rwrite
            | FcallType::Rrename
            | FcallType::Rlink
            | FcallType::Rrenameat
            | FcallType::Runlinkat
            | FcallType::Rlock
            | FcallType::Rgetlock
            | FcallType::Rauth
            | FcallType::Rattach
            | FcallType::Rxattrwalk
            | FcallType::Rxattrcreate
            | FcallType::Rversion
            | FcallType::Rremove
            | FcallType::Rclunk => {
                let tag = u16::from_le_bytes(buf[5..7].try_into().unwrap());
                match self.inflight_tags.remove(&tag) {
                    Some(AttachChange::AddOnSuccess((fid, tattach))) => {
                        self.attach_fids.insert(fid, tattach);
                    }
                    Some(AttachChange::Remove(fid)) => {
                        self.attach_fids.remove(&fid);
                    }
                    Some(AttachChange::None) => (),
                    None => (),
                }
            }
            FcallType::Rlerror => {
                let tag = u16::from_le_bytes(buf[5..7].try_into().unwrap());
                match self.inflight_tags.remove(&tag) {
                    Some(AttachChange::Remove(fid)) => {
                        self.attach_fids.remove(&fid);
                    }
                    // Operation failed, no new attach.
                    Some(AttachChange::AddOnSuccess(_)) => (),
                    Some(AttachChange::None) => (),
                    None => (),
                }
            },
        }
    }
}

fn initial_connect(
    mut client_conn: TcpStream,
    server_addr: String,
) -> Result<ProxyState, std::io::Error> {
    let mut fcall_buf = Vec::with_capacity(8192);

    let tversion = match fcall::read(&mut client_conn, &mut fcall_buf)? {
        fcall::TaggedFcall {
            tag: fcall::NOTAG,
            fcall: Fcall::Tversion(tversion),
        } => tversion.clone_static(),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "9p protocol error, expected Tversion",
            ))
        }
    };

    info!("establishing initial connection to {}", &server_addr);

    let mut first_attempt = true;
    loop {
        if first_attempt {
            first_attempt = false;
        } else {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            error!("retrying initial connection to {}", &server_addr);
        }

        let mut server_conn = match TcpStream::connect(&server_addr) {
            Ok(conn) => conn,
            Err(err) => {
                error!("connection to {} failed: {}", &server_addr, err);
                continue;
            }
        };

        if let Err(err) = fcall::write(
            &mut server_conn,
            &mut fcall_buf,
            &fcall::TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Tversion(tversion.clone_static()),
            },
        ) {
            error!("error writing Tversion to {}: {}", &server_addr, err);
            continue;
        }

        let rversion = match fcall::read(&mut server_conn, &mut fcall_buf)? {
            fcall::TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Rversion(rversion),
            } => rversion.clone_static(),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "9p protocol error, expected Rversion",
                ))
            }
        };

        fcall_buf.resize(rversion.msize as usize, 0);

        fcall::write(
            &mut client_conn,
            &mut fcall_buf,
            &fcall::TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Rversion(rversion.clone()),
            },
        )?;

        return Ok(ProxyState {
            server_state: Arc::new(Mutex::new(ShadowServerState::new())),
            server_addr,
            rversion,
            client_conn,
            server_conn,
        });
    }
}

fn proxy_connection(state: &mut ProxyState) -> Result<(), std::io::Error> {
    let mut fcall_buf = Vec::with_capacity(state.rversion.msize as usize);
    let worker_server_state = state.server_state.clone();

    let (mut worker_client_conn, mut worker_server_conn) =
        match (state.client_conn.try_clone(), state.server_conn.try_clone()) {
            (Ok(client_conn), Ok(server_conn)) => (client_conn, server_conn),
            _ => return Ok(()), // Reconnect and retry.
        };

    let io_worker = std::thread::spawn(move || loop {
        match fcall::read_to_buf(&mut worker_server_conn, &mut fcall_buf) {
            Ok(_) => (),
            Err(err) => {
                debug!("error reading from server: {}", err);
                let _ = worker_server_conn.shutdown(std::net::Shutdown::Both);
                break;
            }
        };

        {
            let mut server_state = worker_server_state.lock().unwrap();
            server_state.on_fcall(&fcall_buf);
        }

        match worker_client_conn.write_all(&fcall_buf) {
            Ok(_) => (),
            Err(err) => {
                debug!("error writing to client: {}", err);
                let _ = worker_server_conn.shutdown(std::net::Shutdown::Both);
                let _ = worker_client_conn.shutdown(std::net::Shutdown::Both);
                break;
            }
        }
    });

    let mut fcall_buf = Vec::with_capacity(state.rversion.msize as usize);

    loop {
        match fcall::read_to_buf(&mut state.client_conn, &mut fcall_buf) {
            Ok(_) => (),
            Err(err) => {
                debug!("error reading from client: {}", err);
                let _ = state.server_conn.shutdown(std::net::Shutdown::Both);
                let _ = state.client_conn.shutdown(std::net::Shutdown::Both);
                io_worker.join().unwrap();
                return Err(err);
            }
        };

        {
            let mut server_state = state.server_state.lock().unwrap();
            server_state.on_fcall(&fcall_buf);
        }

        match state.server_conn.write_all(&fcall_buf) {
            Ok(_) => (),
            Err(err) => {
                debug!("error writing to server: {}", err);
                let _ = state.server_conn.shutdown(std::net::Shutdown::Both);
                io_worker.join().unwrap();
                return Ok(());
            }
        }
    }
}

fn client_eio_until(
    client_conn: &mut TcpStream,
    server_state: &mut ShadowServerState,
    fcall_buf: &mut Vec<u8>,
    delay: time::Duration,
) -> Result<(), std::io::Error> {
    let deadline = time::Instant::now().add(delay);

    loop {
        let now = time::Instant::now();
        if now >= deadline {
            break;
        }

        // Sleep until deadline, handling any incoming packets.
        match fcall::read_to_buf_timeout(client_conn, fcall_buf, deadline - now) {
            Ok(_) => (),
            Err(err) if err.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(err) => return Err(err),
        };

        server_state.on_fcall(fcall_buf);
        let resp = fcall::TaggedFcall {
            tag: u16::from_le_bytes(fcall_buf[5..7].try_into().unwrap()),
            fcall: Fcall::Rlerror(fcall::Rlerror { ecode: lerrno::EIO }),
        };
        resp.encode_to_buf(fcall_buf)?;
        server_state.on_fcall(fcall_buf); // Ensure failed clunks are respected.
        client_conn.write_all(fcall_buf)?;
    }

    client_conn.set_read_timeout(None)?;
    Ok(())
}

fn reconnect(state: &mut ProxyState) -> Result<(), std::io::Error> {
    let mut fcall_buf = Vec::with_capacity(state.rversion.msize as usize);
    let mut server_state = state.server_state.lock().unwrap();

    // Answer remaining in flight calls expecting a response with an error.
    for (tag, _) in server_state.inflight_tags.drain() {
        debug!("cancelling inflight tag {} with EIO", tag);
        fcall::write(
            &mut state.client_conn,
            &mut fcall_buf,
            &fcall::TaggedFcall {
                tag,
                fcall: Fcall::Rlerror(fcall::Rlerror { ecode: lerrno::EIO }),
            },
        )?;
    }

    let mut first_attempt = true;
    loop {
        if first_attempt {
            first_attempt = false;
        } else {
            // While we are disconnected, reply to all requests with EIO.
            client_eio_until(
                &mut state.client_conn,
                &mut server_state,
                &mut fcall_buf,
                time::Duration::from_millis(1000),
            )?;
        }

        info!("attempting to reconnect to {}", &state.server_addr);

        state.server_conn = match TcpStream::connect(&state.server_addr) {
            Ok(conn) => conn,
            Err(err) => {
                error!("reconnect to {} failed: {}", &state.server_addr, err);
                continue;
            }
        };

        // Resend the version, use the rversion we initially got.
        if let Err(err) = fcall::write(
            &mut state.server_conn,
            &mut fcall_buf,
            &fcall::TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Tversion(fcall::Tversion {
                    msize: state.rversion.msize,
                    version: state.rversion.version.clone(),
                }),
            },
        ) {
            error!("writing Tversion to {} failed: {}", &state.server_addr, err);
            continue;
        }

        let rversion = match fcall::read(&mut state.server_conn, &mut fcall_buf)? {
            fcall::TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Rversion(rversion),
            } => rversion.clone_static(),
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "9p protocol error, expected Rversion",
                ))
            }
        };

        if rversion.msize < state.rversion.msize || rversion.version != state.rversion.version {
            // The server has changed it's parameters, we must abort.
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "server resized buffers on reconnect",
            ));
        }

        // Restablish attach fids.
        for (fid, tattach) in server_state.attach_fids.iter() {
            info!(
                "sending Tattach to {} with aname={}",
                state.server_addr, &tattach.aname
            );

            if let Err(err) = fcall::write(
                &mut state.server_conn,
                &mut fcall_buf,
                &fcall::TaggedFcall {
                    tag: fcall::NOTAG,
                    fcall: Fcall::Tattach(fcall::Tattach {
                        fid: *fid,
                        ..tattach.clone()
                    }),
                },
            ) {
                error!("sending Tattach to {} failed: {}", state.server_addr, err);
                continue;
            }

            match fcall::read(&mut state.server_conn, &mut fcall_buf)? {
                fcall::TaggedFcall {
                    tag: fcall::NOTAG,
                    fcall: Fcall::Rattach(_),
                } => (),
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "9p protocol error, expected Rattach",
                    ))
                }
            };
        }

        info!("reconnected to {}", &state.server_addr);
        return Ok(());
    }
}

fn handle_connection(client_conn: TcpStream, server_addr: String) -> Result<(), std::io::Error> {
    let mut state = initial_connect(client_conn, server_addr)?;
    loop {
        proxy_connection(&mut state)?;
        reconnect(&mut state)?;
    }
}

fn usage(program: &str, opts: getopts::Options) {
    let brief = format!(
        "reconnect9 - Proxy 9p connections with automatic reconnection.\n\n\
        Usage: {} --proxy-from --proxy-to",
        program
    );
    print!("{}", opts.usage(&brief));
    std::process::exit(1);
}

fn main() {
    let mut log_builder = env_logger::Builder::new();
    log_builder.filter_level(log::LevelFilter::Info);
    log_builder.parse_env("RECONNECT9_LOG");
    log_builder.init();

    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt(
        "f",
        "from",
        "Proxy 9p connections from this local address (default localhost:5030).",
        "ADDR:PORT",
    );
    opts.optopt(
        "t",
        "to",
        "Proxy 9p connections to this address.",
        "ADDR:PORT",
    );
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            eprintln!("{}: {}", program, err);
            std::process::exit(1)
        }
    };

    if matches.opt_present("h") {
        usage(&program, opts);
    }

    if !matches.opt_present("t") {
        eprintln!("{}: missing --to option", program);
        std::process::exit(1)
    }

    let listen_addr = matches
        .opt_str("from")
        .unwrap_or_else(|| "localhost:5030".to_string());
    let server_addr = matches
        .opt_str("to")
        .unwrap_or_else(|| "localhost:5031".to_string());

    info!("listening on {}, proxying to {}", listen_addr, server_addr);
    let listener = match TcpListener::bind(listen_addr) {
        Ok(l) => l,
        Err(err) => {
            error!("listening failed - {}", err);
            std::process::exit(1)
        }
    };

    for incoming in listener.incoming() {
        let client_conn = incoming.unwrap();
        let server_addr = server_addr.to_string();
        let _ = std::thread::spawn(move || {
            if let Ok(peer) = client_conn.peer_addr() {
                info!("new connection from {}", peer);
            }
            if let Err(err) = handle_connection(client_conn, server_addr) {
                match err.kind() {
                    std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::BrokenPipe
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset => (),
                    _ => error!("connection terminated with error: {}", err),
                }
            };
        });
    }
}
