use p92000::client;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    let conn = TcpStream::connect("127.0.0.1:5030").unwrap();
    let client = client::DotlClient::tcp(conn).unwrap();

    let mut f = client.attach(0, "ac", "/tmp").unwrap();

    let _ = f.write(&[1, 2, 3]);

    println!("dropping client...");
    drop(client);
    println!("dropping file...");
    drop(f);
    println!("good bye!");
}
