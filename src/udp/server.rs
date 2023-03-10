use byte_unit::Byte;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Cursor, Read, Write};
use std::net::UdpSocket;
use std::{thread, time::Duration};

fn receive_file_from_socket(socket: &mut UdpSocket) -> io::Result<String> {
    let mut buf = vec![];
    loop {
        match socket.recv() {
            _ => todo!(),
        }
    }
}

fn handle_received(buf: &[u8]) -> io::Result<String> {
    let mut buf = Vec::new();
    loop {
        match stream.read_to_end(&mut buf) {
            Ok(_) => break,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("Waiting for more packets");
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        };
    }
    let mut cursor = Cursor::new(buf);
    let file_name_size = cursor.read_u32::<BigEndian>()? as usize;
    println!("Read size {file_name_size}");
    let file_name = {
        let mut v = vec![0; file_name_size];
        cursor.read_exact(&mut v)?;
        let name = String::from_utf8(v).expect("Should receive valid UTF-8 from client");
        format!("{}_out", name)
    };
    println!("Read file name {file_name}");
    let file_size = cursor.read_u32::<BigEndian>()? as usize;
    let bytes = Byte::from_bytes(file_size as u128).get_appropriate_unit(true);
    println!("Read file content size {file_size} which is {bytes}");
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&file_name)?;
    let mut file = BufWriter::new(file);
    let contents = {
        let mut v = vec![0; file_size];
        cursor.read_exact(&mut v)?;
        v
    };
    println!("Writing contents to {file_name}");
    file.write(&contents)?;
    println!("Wrote contents to {file_name} successfully");

    Ok(file_name.to_string())
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
    let socket = UdpSocket::bind(&addr).expect(&format!("Could bind to port {}", port));
    println!("Succesfully bound to {addr}!");

    loop {
        match receive_file_from_socket(&mut socket) {
            Ok(fname) => println!("Read a file to {fname}"),
            Err(e) => eprintln!("Failed reading a file! {e}"),
        }
    }
}
