use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::str::from_utf8;

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

fn receive_request<T>(mut stream: T)
    where T: Read
{
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

fn make_request<T>(mut stream: T)
    where T: Write
{
    stream.write(&[1]).unwrap();
}

fn connect(addr: &str) -> TcpStream {
    return TcpStream::connect(addr).unwrap();
}

pub fn download_files(files: &Vec<&str>) {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
    for file in files {
        pool.install(move || {
            let stream = connect(file);
            make_request(&stream);
            receive_request(&stream);
        });
    }
}