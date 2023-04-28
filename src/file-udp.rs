use byteorder::{BigEndian, ReadBytesExt};
use std::env;
use std::fs;
use std::io::{self, Cursor, Read};
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

use extended::common::{log, SLEEP_MS};

fn become_receiver(mut args: env::Args) -> io::Result<()> {
    let port = args
        .next()
        .expect("Expected a second argument")
        .parse::<usize>()
        .expect("Provided a number as first argument");
    let addr = format!("127.0.0.1:{port}");
    let socket = UdpSocket::bind(&addr)?;
    socket.set_nonblocking(true)?;
    println!("Bound to {addr}");

    let read_from_socket = || -> io::Result<(usize, Vec<u8>)> {
        let mut buf = [0; 10 * 1024]; // 10 KB
        log!("Reading from socket");
        let (bytes_read, _) = loop {
            match socket.recv_from(&mut buf) {
                Ok(n) => break n,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    log!("Waiting");
                    thread::sleep(Duration::from_millis(SLEEP_MS));
                    Ok(())
                }
                Err(e) => Err(e),
            }?
        };
        log!("Read {bytes_read} bytes");
        Ok((bytes_read, buf[..bytes_read].to_vec()))
    };

    let extend_with_read = |buf: &mut Vec<u8>| -> io::Result<bool> {
        log!("Extending buffer with curr len {}", buf.len());
        let mut retries = 5;
        let (mut read_bytes, mut read_buf) = read_from_socket()?;
        while retries != 0 && read_bytes == 0 {
            log!("Retry number {}", 5 - retries);
            (read_bytes, read_buf) = read_from_socket()?;
            thread::sleep(Duration::from_millis(SLEEP_MS));
            retries -= 1;
        }

        Ok(if retries == 0 {
            false
        } else {
            buf.extend(read_buf);
            true
        })
    };

    let extend_at_least = |buf: &mut Vec<u8>, n: usize| -> io::Result<()> {
        let len = buf.len();
        while buf.len() < len + n {
            if !extend_with_read(buf)? {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Missing data!",
                ));
            }
        }
        Ok(())
    };

    let mut buf = vec![];
    extend_at_least(&mut buf, 8)?;
    let len = buf.as_slice().read_u64::<BigEndian>()? as usize;
    println!("Read len {len}");
    let now = Instant::now();
    println!("Length before extending: {}", buf.len());
    extend_at_least(&mut buf, len + 4)?;
    println!("Took {:?} to receive the data", now.elapsed());
    let mut cursor = Cursor::new(&buf[8..]);
    let file = {
        let mut file_buf = vec![0; len];
        cursor.read_exact(&mut file_buf)?;
        file_buf
    };
    let recv_hash = cursor.read_u32::<BigEndian>()?;
    println!("Got hash {recv_hash}");
    let computed_hash = crc32fast::hash(&file);
    println!("Computed hash {computed_hash}");
    if recv_hash != computed_hash {
        println!("Hashes differ, something went wrong!");
    } else {
        println!("Hashes match, file sent successfully!");
    }

    Ok(())
}

fn become_sender(mut args: env::Args) -> io::Result<()> {
    let addr = args.next().expect("Expected a second argument");
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect(&addr)?;
    println!("Connected to {addr}");

    let fname = args.next().expect("Expected a third argument");
    let file = fs::read(fname)?;
    let len = file.len() as u64;
    let hash = crc32fast::hash(&file);
    println!("Sending len {len}");
    println!("Sending hash {hash}");

    socket.send(&len.to_be_bytes())?;
    socket.send(&file)?;
    socket.send(&hash.to_be_bytes())?;
    println!("Finished sending files to socket!");

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
