use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

async fn receive_request(stream: &mut TcpStream)
{
    const BUFF_SIZE: usize = 128;

    let mut buff = [0; BUFF_SIZE];
    let mut request_str = String::new();
    loop {
        let bits_read = stream.read(&mut buff).await.unwrap();
        request_str += from_utf8(&buff).unwrap();

        if bits_read < BUFF_SIZE {
            break;
        }
    }

    parse_request(&request_str);
}

async fn make_request(stream: &mut TcpStream)
{
    stream.writable().await.unwrap();
    stream.write(&[1]).await.unwrap();
}

async fn connect(addr: &str) -> TcpStream {
    let tcp_stream = TcpStream::connect(addr).await.unwrap();
    tcp_stream
}

async fn download_one_file(address: &str) {
    let mut stream = connect(address).await;
    make_request(&mut stream).await;
    receive_request(&mut stream).await;
}

pub async  fn download_files(files: &Vec<&str>) {
    let mut futures = vec![];
    for file in files {
        let future =download_one_file(file);
        futures.push(future);
    }

    for el in futures {
        el.await;
    }
}