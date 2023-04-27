use std::env;
use std::io;
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant};

use extended::common::{log, SimplePayload, MAX_LOOPS, WAITING_MS};

fn become_sender(addr: &str) -> io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect(addr)?;
    println!("Connected to {addr}");

    for _ in 0..MAX_LOOPS {
        let payload = SimplePayload::new();
        let payload = payload.to_string();
        print!("Sending {payload}...");
        socket.send(payload.as_bytes())?;
        println!("Sent!");
    }

    Ok(())
}

fn become_receiver(port: &str) -> io::Result<()> {
    let port = port
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
                    thread::sleep(Duration::from_millis(WAITING_MS));
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
            thread::sleep(Duration::from_millis(100));
            retries -= 1;
        }

        Ok(if retries == 0 {
            false
        } else {
            buf.extend(read_buf);
            true
        })
    };

    let mut buf = vec![];
    let mut read_until = 0;
    let mut read_payloads = 0;
    let mut invalid_payloads = 0;
    let mut now = Instant::now();
    while read_payloads < MAX_LOOPS {
        log!("Starting loop");
        if !extend_with_read(&mut buf)? {
            println!("Finished reading!");
            break;
        }
        now = Instant::now();
        log!("Finished first if");
        let curr_buf = &buf[read_until..];

        let idx = match curr_buf.iter().enumerate().find(|(_, v)| **v == b'}') {
            Some((idx, _)) => idx,
            None => {
                log!("Could not find {{");
                if !extend_with_read(&mut buf)? {
                    log!("Finished reading!");
                    break;
                } else {
                    continue;
                }
            }
        };

        let payload_buf = &curr_buf[..=idx];
        let mut payload: serde_json::Result<SimplePayload> = serde_json::from_reader(payload_buf);
        while payload.is_err() {
            log!("Invalid payload");
            if !extend_with_read(&mut buf)? {
                log!("Finished reading");
                break;
            }
            let payload_buf = &buf[read_until..=idx];
            payload = serde_json::from_reader(payload_buf);
        }
        let payload = payload.unwrap();
        println!("Read {}", payload.to_string());
        if payload.x != payload.millis / 60 && payload.y != payload.millis / (60 * 60) {
            println!("Read invalid payload!");
            invalid_payloads += 1;
        }
        read_payloads += 1;
        read_until += idx + 1;
    }
    println!("Time elapsed: {:?}", now.elapsed());
    println!(
        "{}/{} were read correctly!",
        read_payloads - invalid_payloads,
        read_payloads
    );
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
