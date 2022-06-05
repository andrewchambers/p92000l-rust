use p92000::fcall::LOpenFlags;
use p92000::lclient;
// use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let conn = TcpStream::connect("127.0.0.1:5030").unwrap();
    let client = lclient::Client::tcp(conn, 128 * 1024).unwrap();

    let (_, f) = client.attach(0, "ac", "/tmp").unwrap();
    f.open(LOpenFlags::O_RDONLY).unwrap();
    dbg!(f.read_dir().unwrap());
    f.clunk().unwrap();

    println!("good bye!");
}
