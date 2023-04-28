use byteorder::{BigEndian, ReadBytesExt};
use std::env;
use std::fs;
use std::io::{self, Cursor, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use extended::common::SLEEP_MS;

fn handle_incoming(mut stream: TcpStream) -> io::Result<()> {
    let now = Instant::now();
    println!("Received input from {}", stream.peer_addr()?);
    stream.set_nonblocking(true)?;
    let mut buf = vec![];
    loop {
        match stream.read_to_end(&mut buf) {
            Ok(_) => break,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                println!("Waiting");
                thread::sleep(Duration::from_millis(SLEEP_MS));
            }
            Err(e) => return Err(e),
        }
    }
    println!("Took {:?} to receive the data", now.elapsed());

    let buf_len = buf.len();
    let mut cursor = Cursor::new(buf);
    let len = cursor.read_u64::<BigEndian>()? as usize;
    println!("Got len {len}");
    let file = {
        let mut buf = vec![0; len];
        match cursor.read_exact(&mut buf) {
            Ok(_) => {} // don't care
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                eprintln!("Did not receive enough bytes!");
                eprintln!(
                    "Only got {} when at least {} where needed!",
                    buf_len - 8,
                    len
                );
                return Err(e);
            }
            e @ Err(_) => return e,
        }
        buf
    };

    let recv_hash = cursor.read_u32::<BigEndian>()?;
    println!("Received hash {recv_hash}");
    let computed_hash = crc32fast::hash(&file);
    println!("Computed hash {computed_hash}");
    if recv_hash != computed_hash {
        println!("Hashes differ, something went wrong!");
    } else {
        println!("Hashes match, file sent successfully!");
    }
    Ok(())
}

fn become_receiver(mut args: env::Args) -> io::Result<()> {
    let port = args
        .next()
        .expect("Provided a second argument")
        .parse::<usize>()
        .expect("Provided a number as second argument");
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)?;
    println!("Bound to {addr}");

    for stream in listener.incoming() {
        handle_incoming(stream?)?;
    }
    Ok(())
}

fn become_sender(mut args: env::Args) -> io::Result<()> {
    let addr = args.next().expect("Provided a second argument");
    let fname = args.next().expect("Provided a third argument");
    let file = fs::read(&fname)?;
    let file_len = file.len();
    println!("Read file {fname} of length {} bytes", file_len);

    println!("Trying to connect to {addr}");
    let mut stream = TcpStream::connect(&addr)?;
    println!("Connected to {addr}");

    stream.write(&(file_len as u64).to_be_bytes())?;
    stream.write(&file)?;
    let hash = crc32fast::hash(&file);
    println!("Sending hash {hash}");
    stream.write(&hash.to_be_bytes())?;
    println!("Finished sending!");

    Ok(())
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    let _name = args.next().expect("There must be a 0th argument");
    let ty = args.next().expect("Provided a first argument");

    match ty.as_str() {
        "sender" => become_sender(args),
        "receiver" => become_receiver(args),
        _ => panic!("Unknown program type {}", ty),
    }
}
