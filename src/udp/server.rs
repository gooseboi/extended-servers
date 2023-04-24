use byte_unit::Byte;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Cursor, Read, Write};
use std::net::UdpSocket;
use std::{thread, time::Duration};

const MAX_DATAGRAM_SIZE: usize = 64 * 1024;

fn receive_file_from_socket(socket: &mut UdpSocket) -> io::Result<String> {
    let mut buf = Vec::new();
    let read_again = |buf: &mut Vec<u8>| -> io::Result<Cursor<Vec<u8>>> {
        let mut read_buf = [0; MAX_DATAGRAM_SIZE];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((read_bytes, _src)) => {
                    let read = &read_buf[..read_bytes];
                    buf.extend(read);
                    break;
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    println!("Waiting for more packets");
                    thread::sleep(Duration::from_millis(100));
                }
                e@Err(_) => return e,
            };
        }
        Ok(Cursor::new(buf.to_owned()))
    };
    let read_until_enough = |buf: &mut Vec<u8>, bytes: usize| -> io::Result<Cursor<Vec<u8>>>{
        while buf.len() < bytes {
            read_again(buf)?;
        }
        Ok(Cursor::new(buf.to_owned()))
    };
    let c = read_until_enough(&mut buf, 4)?;
    let file_name_size = c.read_u32::<BigEndian>()? as usize;
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
    let mut socket = UdpSocket::bind(&addr).expect(&format!("Could bind to port {}", port));
    println!("Succesfully bound to {addr}!");

    loop {
        match receive_file_from_socket(&mut socket) {
            Ok(fname) => println!("Read a file to {fname}"),
            Err(e) => eprintln!("Failed reading a file! {e}"),
        }
    }
}
