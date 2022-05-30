use p92000::client;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let conn = TcpStream::connect("127.0.0.1:5030").unwrap();
    let client = client::DotlClient::tcp(conn, 8912).unwrap();

    let mut f = client.attach(0, "ac", "/tmp").unwrap();

    f.open(0).unwrap();

    dbg!(f.read_dir().unwrap());

    f.close().unwrap();

    println!("good bye!");
}
