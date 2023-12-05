use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::str::from_utf8;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

fn parse_request(request_str: &str) {
    let lines: Vec<String> = request_str.lines().map(|line| line.to_string()).collect();

    for line in lines {
        if line.contains("Content-Length: ") {
            let elements: Vec<&str> = line.split(":").collect();
            let content_length = elements.get(1).unwrap()
                .trim()
                .parse::<i32>()
                .unwrap();

            println!("Content length is {}", content_length);
            return;
        }
    }
}

fn receive_request<T>(stream_channel: Receiver<T>)
    where T: Read
{
    let mut stream = stream_channel.recv().unwrap();
    const BUFF_SIZE: usize = 128;

    let mut buff = [0; BUFF_SIZE];
    let mut request_str = String::new();
    loop {
        let bits_read = stream.read(&mut buff).unwrap();
        request_str += from_utf8(&buff).unwrap();

        if bits_read < BUFF_SIZE {
            break;
        }
    }

    parse_request(&request_str);
}

fn make_request<T>(stream_channel: Receiver<T>, sender_channel: Sender<T>)
    where T: Write
{
    let mut stream = stream_channel.recv().unwrap();
    stream.write(&[1]).unwrap();
    sender_channel.send(stream).unwrap();
}

fn connect(addr: &str, channel: Sender<TcpStream>) {
    let tcp_stream = TcpStream::connect(addr).unwrap();
    channel.send(tcp_stream).unwrap();
}

pub fn download_files(files: &Vec<&str>) {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(5).build().unwrap();
    for file in files {
        let (connect_tx, request_rx) = mpsc::channel::<TcpStream>();
        let (request_tx, receive_rx) = mpsc::channel::<TcpStream>();
        pool.install(move || {
            connect(file, connect_tx);
        });
        pool.install(move || {
            make_request(request_rx, request_tx);
        });
        pool.install(move || {
            receive_request(receive_rx);
        });
    }
}