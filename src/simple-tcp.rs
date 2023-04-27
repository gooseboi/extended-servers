use std::env;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use extended::common::{SimplePayload, MAX_LOOPS, WAITING_MS, SLEEP_MS};

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

    let mut read = &buf[..];
    loop {
        let idx = match buf.iter().enumerate().find(|(_, v)| **v == b'}') {
            Some((idx, _)) => idx,
            None => break,
        };
        let payload: serde_json::Result<SimplePayload> = serde_json::from_reader(&read[..=idx]);
        println!("Received payload {payload:?}");
        if read.len() <= idx + 1 {
            break;
        };
        read = &read[(idx + 1)..];
    }
    println!("Finished reading!");
    println!("Took {:?}", now.elapsed());

    Ok(())
}

fn become_receiver(port: &str) -> io::Result<()> {
    let port = port
        .parse::<usize>()
        .expect("Provided a number as first argument");
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)?;
    println!("Bound to {addr}");

    for stream in listener.incoming() {
        handle_incoming(stream?)?;
    }
    Ok(())
}

fn become_sender(addr: &str) -> io::Result<()> {
    println!("Trying to connect to {addr}");
    let mut stream = TcpStream::connect(&addr)?;
    println!("Connected to {addr}");
    for i in 0..MAX_LOOPS {
        let payload = SimplePayload::new();
        let s = payload.to_string();
        print!("Sending {s}...");
        match stream.write(s.as_bytes()) {
            Ok(_) => {
                println!("Sent!");
            }
            e @ Err(_) => eprintln!("Could only send {i} copies: {e:?}"),
        }
        thread::sleep(Duration::from_millis(WAITING_MS));
    }
    println!("Finished sending!");

    Ok(())
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    let _name = args.next().expect("There must be a 0th argument");
    let ty = args.next().expect("Provided a first argument");

    let snd = args.next().expect("Provided a second argument");
    match ty.as_str() {
        "sender" => become_sender(&snd),
        "receiver" => become_receiver(&snd),
        _ => panic!("Unknown program type {}", ty),
    }
}
