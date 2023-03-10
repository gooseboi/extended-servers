use byte_unit::Byte;
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

fn send_file(stream: &mut impl Write, file_name: &str, mut file: File) -> io::Result<()> {
    let mut contents = vec![];
    file.read_to_end(&mut contents)?;
    let checksum = crc32fast::hash(&contents);
    let fsize = file_name.len() as u32;
    stream.write(&(fsize).to_be_bytes())?;
    println!("Sent size {fsize}");
    stream.write(file_name.as_bytes())?;
    println!("Sent file name {file_name}");
    let content_length = contents.len() as u32;
    stream.write(&(content_length).to_be_bytes())?;
    stream.write(&checksum.to_be_bytes());
    println!("Sent checksum {checksum}");
    let bytes = Byte::from_bytes(content_length.into()).get_appropriate_unit(true);
    println!("Sent content length {content_length} which is {bytes}");
    stream.write(&contents)?;

    Ok(())
}

fn main() {
    let mut args = std::env::args();
    let _name = args.next().expect("There must be a 0th argument");
    let addr = args
        .next()
        .expect("Provided a first argument.")
        .to_socket_addrs()
        .expect("Provided a valid address as first argument")
        .next()
        .expect("Provided at least one address");

    let stream = TcpStream::connect(&addr).expect(&format!("Could connect to address {}", addr));
    let mut stream = BufWriter::new(stream);
    println!("Succesfully connected to {addr}!");

    let file_name = args
        .next()
        .expect("Provided a file name as the second argument");
    let file = File::open(&file_name).expect("Failed opening file");
    match send_file(&mut stream, &file_name, file) {
        Ok(_) => println!("Successfully sent file {file_name} to {addr}"),
        Err(e) => eprintln!("Failed sending a file! {}", e),
    }
}
