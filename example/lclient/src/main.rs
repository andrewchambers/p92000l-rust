use p92000l;
use p92000l::LOpenFlags;
use std::net::TcpStream;

fn main() {
    let conn = TcpStream::connect("127.0.0.1:5030").unwrap();
    let client = p92000l::Client::tcp(conn, 128 * 1024).unwrap();
    let (_, f) = client.attach(0, "ac", "/tmp").unwrap();
    f.open(LOpenFlags::O_RDONLY).unwrap();
    dbg!(f.read_dir().unwrap());
    println!("good bye!");
}
