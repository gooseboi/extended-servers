use extended::common::{SimplePayload, MAX_LOOPS};
use serde_json::error::Category;
use std::env;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::time::Duration;

fn become_client(mut args: env::Args) -> io::Result<()> {
    let addr = args
        .next()
        .expect("Provided a second argument.")
        .to_socket_addrs()
        .expect("Provided a valid address as second argument")
        .next()
        .expect("Provided at least one address");
    let mut stream = TcpStream::connect(&addr).expect(&format!("Could connect to address {}", addr));
    //let mut buf_stream = BufWriter::new(&stream);
    println!("Succesfully connected to {addr}!");

    for i in 0..MAX_LOOPS {
        let payload = SimplePayload::new();
        let s = payload.to_string();
        print!("Sending {s}...");
        //match stream.write(s.as_bytes()) {
            //Ok(n) => {
                //println!("Sent! {n}");
            //}
            //e @ Err(_) => eprintln!("Could only send {i} copies: {e:?}"),
        //}
        thread::sleep(Duration::from_millis(1));
    }
    let payload = SimplePayload::new();
    let s = payload.to_string();
    stream.write(s.as_bytes())?;
    let payload = SimplePayload::new();
    let s = payload.to_string();
    stream.write(s.as_bytes())?;
    let payload = SimplePayload::new();
    let s = payload.to_string();
    stream.write(s.as_bytes())?;
    let payload = SimplePayload::new();
    let s = payload.to_string();
    stream.write(s.as_bytes())?;
    //buf_stream.flush()?;
    Ok(())
}

fn handle_incoming(stream: TcpStream) -> io::Result<usize> {
    let mut stream = BufReader::new(stream);
    let mut buf = vec![];
    let stream = &mut stream;
    //let mut read_again = |buf: &mut Vec<u8>| -> io::Result<()> {
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
    //};
    let mut read = 0usize;
    let mut nth = 0usize;
    loop {
        if nth == MAX_LOOPS {
            break Ok(read);
        }
        //read_again(&mut buf)?;

        let idx = match buf
            .iter()
            .enumerate()
            .take_while(|(_, c)| **c != b'}')
            .last()
        {
            Some(n) => n,
            None => continue,
        }
        .0 + 1;
        let read_from = &buf[..=idx];

        let res: serde_json::Result<SimplePayload> = serde_json::from_reader(&read_from[..]);
        match res {
            Ok(payload) => {
                println!("Read {payload:?}");
                if payload.x == payload.millis / 60 && payload.y == payload.millis / (60 * 60) {
                    println!("Valid");
                    read += 1;
                } else {
                    println!("Invalid");
                }
                buf = buf.iter().skip(idx).copied().collect();
            }
            Err(e) if e.classify() == Category::Eof => {
                continue;
            }
            Err(e) => {
                let to_str = |s: &[u8]| String::from_utf8(s.to_vec()).unwrap();
                eprintln!(
                    "Failed reading {nth}: {e:?}: {}: {}",
                    to_str(&buf[..]),
                    to_str(&read_from[..])
                );
            }
        };
        nth += 1;
    }
}

fn become_server(mut args: env::Args) -> io::Result<()> {
    let port = args
        .next()
        .expect("Provided a second argument.")
        .parse::<usize>()
        .expect("Provided a valid port as first argument");

    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).expect(&format!("Could bind to port {}", port));
    println!("Succesfully bound to {addr}!");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let n = handle_incoming(stream)?;
                println!("Read {} copies from origin!", n);
            }
            Err(e) => eprintln!("Failed connection! {:?}", e),
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    let _name = args.next().expect("There must be a 0th argument");
    let ty = args.next().expect("Provided a first argument");

    match ty.as_str() {
        "client" => become_client(args),
        "server" => become_server(args),
        _ => panic!("Unknown program type {}", ty),
    }
}
