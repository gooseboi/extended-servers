use byte_unit::Byte;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self,  Cursor, Read};
use std::net::{TcpListener, TcpStream};
use std::{thread, time::Duration};

fn handle_stream() -> usize {

}

fn main() {
    let mut args = std::env::args();
    let _name = args.next().expect("There must be a 0th argument");
    let port = args
        .next()
        .expect("Provided a first argument.")
        .parse::<usize>()
        .expect("Provided a valid port as first argument");

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).expect(&format!("Could bind to port {}", port));
    println!("Succesfully bound to {addr}!");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let n = handle_stream(stream);
                println!("Read {} copies from origin!");
            }
            Err(e) => eprintln!("Failed connection! {:?}", e),
        }
    }
}
