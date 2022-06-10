use log::{debug, error, info, log_enabled, Level};
use p92000::fcall;
use p92000::fcall::Fcall;
use p92000::lerrno;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::Mutex;

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

    fn on_fcall(&mut self, tagged_fcall: &fcall::TaggedFcall) {
        match &tagged_fcall.fcall {
            Fcall::Tattach(tattach) => {
                self.inflight_tags.insert(
                    tagged_fcall.tag,
                    AttachChange::AddOnSuccess((tattach.fid, tattach.clone_static())),
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
            Fcall::Twalk(_) // For now we simply ignore clones.
            | Fcall::Tstatfs(_)
            | Fcall::Tlopen(_)
            | Fcall::Tlcreate(_)
            | Fcall::Tsymlink(_)
            | Fcall::Tmknod(_)
            | Fcall::Treadlink(_)
            | Fcall::Tgetattr(_)
            | Fcall::Tsetattr(_)
            | Fcall::Treaddir(_)
            | Fcall::Tfsync(_)
            | Fcall::Tmkdir(_)
            | Fcall::Tflush(_)
            | Fcall::Tread(_)
            | Fcall::Twrite(_)
            | Fcall::Trename(_)
            | Fcall::Tlink(_)
            | Fcall::Trenameat(_)
            | Fcall::Tunlinkat(_)
            | Fcall::Tlock(_)
            | Fcall::Tgetlock(_)
            | Fcall::Tauth(_)
            | Fcall::Txattrwalk(_)
            | Fcall::Txattrcreate(_)
            | Fcall::Tversion(_) => {
                self
                .inflight_tags
                .insert(tagged_fcall.tag, AttachChange::None);
            }
             Fcall::Rwalk(_)
            | Fcall::Rstatfs(_)
            | Fcall::Rlopen(_)
            | Fcall::Rlcreate(_)
            | Fcall::Rsymlink(_)
            | Fcall::Rmknod(_)
            | Fcall::Rreadlink(_)
            | Fcall::Rgetattr(_)
            | Fcall::Rsetattr(_)
            | Fcall::Rreaddir(_)
            | Fcall::Rfsync(_)
            | Fcall::Rmkdir(_)
            | Fcall::Rflush(_)
            | Fcall::Rread(_)
            | Fcall::Rwrite(_)
            | Fcall::Rrename(_)
            | Fcall::Rlink(_)
            | Fcall::Rrenameat(_)
            | Fcall::Runlinkat(_)
            | Fcall::Rlock(_)
            | Fcall::Rgetlock(_)
            | Fcall::Rauth(_)
            | Fcall::Rattach(_)
            | Fcall::Rxattrwalk(_)
            | Fcall::Rxattrcreate(_)
            | Fcall::Rversion(_)
            | Fcall::Rremove(_)
            | Fcall::Rclunk(_) => match self.inflight_tags.remove(&tagged_fcall.tag) {
                Some(AttachChange::AddOnSuccess((fid, tattach))) => {
                    self.attach_fids.insert(fid, tattach);
                }
                Some(AttachChange::Remove(fid)) => {
                    self.attach_fids.remove(&fid);
                }
                Some(AttachChange::None) => (),
                None => (),
            },
            Fcall::Rlerror(_) => match self.inflight_tags.remove(&tagged_fcall.tag) {
                Some(AttachChange::Remove(fid)) => {
                    self.attach_fids.remove(&fid);
                }
                // Operation failed, no new attach.
                Some(AttachChange::AddOnSuccess(_)) => (),
                Some(AttachChange::None) => (),
                None => (),
            },
        }
    }
}

fn initial_connect(
    mut client_conn: TcpStream,
    server_addr: String,
) -> Result<ProxyState, std::io::Error> {
    let mut bufsize = 8192;
    let mut rbuf = Vec::with_capacity(bufsize);
    let mut wbuf = Vec::with_capacity(bufsize);

    let tversion = match fcall::read(&mut client_conn, &mut rbuf)? {
        fcall::TaggedFcall {
            tag: fcall::NOTAG,
            fcall: Fcall::Tversion(tversion),
        } => tversion,
        _ => todo!(),
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
            Err(_) => continue,
        };

        if fcall::write(
            &mut server_conn,
            &mut wbuf,
            &fcall::TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Tversion(tversion.clone_static()),
            },
        )
        .is_err()
        {
            continue;
        }

        let rversion = match fcall::read(&mut server_conn, &mut rbuf)? {
            fcall::TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Rversion(rversion),
            } => rversion.clone_static(),
            _ => todo!(),
        };

        bufsize = rversion.msize as usize;
        rbuf.resize(bufsize, 0);
        wbuf.resize(bufsize, 0);

        fcall::write(
            &mut client_conn,
            &mut wbuf,
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
    let mut rbuf = Vec::with_capacity(state.rversion.msize as usize);
    let mut wbuf = Vec::with_capacity(state.rversion.msize as usize);
    let worker_server_state = state.server_state.clone();

    let (mut worker_client_conn, mut worker_server_conn) =
        match (state.client_conn.try_clone(), state.server_conn.try_clone()) {
            (Ok(client_conn), Ok(server_conn)) => (client_conn, server_conn),
            _ => return Ok(()), // Reconnect and retry.
        };

    let io_worker = std::thread::spawn(move || loop {
        let tagged_fcall = match fcall::read(&mut worker_server_conn, &mut rbuf) {
            Ok(tagged_fcall) => tagged_fcall,
            Err(err) => {
                error!("error reading from server: {}", err);
                let _ = worker_server_conn.shutdown(std::net::Shutdown::Both);
                break;
            }
        };

        {
            let mut server_state = worker_server_state.lock().unwrap();
            server_state.on_fcall(&tagged_fcall);
        }

        match fcall::write(&mut worker_client_conn, &mut wbuf, &tagged_fcall) {
            Ok(_) => (),
            Err(err) => {
                error!("error writing to client: {}", err);
                let _ = worker_server_conn.shutdown(std::net::Shutdown::Both);
                let _ = worker_client_conn.shutdown(std::net::Shutdown::Both);
                break;
            }
        }
    });

    let mut rbuf = Vec::with_capacity(state.rversion.msize as usize);
    let mut wbuf = Vec::with_capacity(state.rversion.msize as usize);

    loop {
        let tagged_fcall = match fcall::read(&mut state.client_conn, &mut rbuf) {
            Ok(tagged_fcall) => tagged_fcall,
            Err(err) => {
                error!("error reading from client: {}", err);
                let _ = state.server_conn.shutdown(std::net::Shutdown::Both);
                let _ = state.client_conn.shutdown(std::net::Shutdown::Both);
                io_worker.join().unwrap();
                return Err(err);
            }
        };

        {
            let mut server_state = state.server_state.lock().unwrap();
            server_state.on_fcall(&tagged_fcall);
        }

        match fcall::write(&mut state.server_conn, &mut wbuf, &tagged_fcall) {
            Ok(_) => (),
            Err(err) => {
                error!("error writing to server: {}", err);
                let _ = state.server_conn.shutdown(std::net::Shutdown::Both);
                io_worker.join().unwrap();
                return Ok(());
            }
        }
    }
}

fn reconnect(state: &mut ProxyState) -> Result<(), std::io::Error> {
    let mut rbuf = Vec::with_capacity(state.rversion.msize as usize);
    let mut wbuf = Vec::with_capacity(state.rversion.msize as usize);

    let mut server_state = state.server_state.lock().unwrap();

    // Answer remaining in flight calls expecting a response with an error.
    for (tag, _) in server_state.inflight_tags.drain() {
        debug!("cancelling inflight tag {} with EIO", tag);
        fcall::write(
            &mut state.client_conn,
            &mut wbuf,
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
            std::thread::sleep(std::time::Duration::from_millis(1000));
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
            &mut wbuf,
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

        let rversion = match fcall::read(&mut state.server_conn, &mut rbuf)? {
            fcall::TaggedFcall {
                tag: fcall::NOTAG,
                fcall: Fcall::Rversion(rversion),
            } => rversion.clone_static(),
            _ => todo!(),
        };

        if rversion.msize < state.rversion.msize || rversion.version != state.rversion.version {
            // The server has changed it's parameters, we must abort.
            todo!()
        }

        // Restablish attach fids.
        for (fid, tattach) in server_state.attach_fids.iter() {
            info!(
                "sending Tattach to {} with aname={}",
                state.server_addr, &tattach.aname
            );

            if let Err(err) = fcall::write(
                &mut state.server_conn,
                &mut wbuf,
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

            match fcall::read(&mut state.server_conn, &mut rbuf)? {
                fcall::TaggedFcall {
                    tag: fcall::NOTAG,
                    fcall: Fcall::Rattach(_),
                } => (),
                _ => todo!(),
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
        .unwrap_or("localhost:5030".to_string());
    let server_addr = matches
        .opt_str("to")
        .unwrap_or("localhost:5031".to_string());

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
            let _ = handle_connection(client_conn, server_addr);
        });
    }
}
