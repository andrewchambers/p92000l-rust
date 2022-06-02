use p92000::client;
use p92000::fcall;
use p92000::fcall::LOpenFlags;
// use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let conn = TcpStream::connect("127.0.0.1:5030").unwrap();
    let client = client::DotlClient::tcp(conn, 8912).unwrap();

    let (_, f) = client.attach(0, "ac", "/tmp").unwrap();

    dbg!(f.getattr(fcall::GetattrMask::empty()).unwrap());
    dbg!(f.getattr(fcall::GetattrMask::all()).unwrap());

    f.open(LOpenFlags::O_RDONLY).unwrap();

    let (_, f) = f.walk(&["derp"]).unwrap();
    f.open(LOpenFlags::O_RDONLY).unwrap();
    // dbg!(f.read_dir().unwrap());
    f.clunk().unwrap();

    println!("good bye!");
}
