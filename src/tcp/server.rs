use byte_unit::Byte;
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{self,  Cursor, Read};
use std::net::{TcpListener, TcpStream};
use std::{thread, time::Duration};

fn handle_stream(mut stream: TcpStream) -> io::Result<(u32, u32)> {
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
    let checksum = cursor.read_u32::<BigEndian>()?;
    println!("Got checksum {checksum}");
    let contents = {
        let mut v = vec![0; file_size];
        cursor.read_exact(&mut v)?;
        v
    };
    let new_checksum = crc32fast::hash(&contents);
    Ok((checksum, new_checksum))
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
                match handle_stream(stream) {
                    Ok((checksum, new_checksum)) if checksum == new_checksum => {
                        eprintln!("Checksum match failed!");
                        eprintln!("Source checksum was {checksum}");
                        eprintln!("We got checksum {new_checksum}");
                    }
                    Ok(_) => println!("Successfully received file!"),
                    Err(e) => eprintln!("Failed reading from stream: {}", e),
                };
            }
            Err(e) => eprintln!("Failed connection! {:?}", e),
        }
    }
}
