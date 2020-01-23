use std::{fs, thread};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use webserver::ThreadPool;

struct response_data {
    protocol: String,
    status_header: String,
    body: String,
}

fn construct_response_string(seed: &response_data, content: &str) -> String {
    /*
       **TCP response format**

       HTTP-Version Status-Code Reason-Phrase CRLF
       headers CRLF
       message-body
       
       eg:
          PUT /test1%2Epdf HTTP/1.1
          Authorization: Basic xxxx
          User-Agent: curl/7.20.0 (i386-pc-win32) libcurl/7.20.0 OpenSSL/0.9.8l zlib/1.2.3
          Host: localhost
          Accept: *//*
          Content-Length: 24
          Expect: 100-continue
   */
    format!(
        "{} {}\r\n\r\n{}",
        seed.protocol,
        seed.status_header,
        content
    )
}

fn connection_handler(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer);

    let mut raw_response_data = response_data {
        protocol: String::from("HTTP/1.1"),
        status_header: String::from("200 OK"),
        body: String::new(),
    };
    let html_response_body: String = fs::read_to_string(
        if buffer.starts_with(b"GET / ") {
            "webapp/index.html"
        } else {
            raw_response_data.status_header = String::from("404 NOT FOUND");
            "webapp/404.html"
        }
    ).unwrap();

    let tcp_response = construct_response_string(&raw_response_data, &html_response_body);
    thread::sleep(std::time::Duration::from_secs(5));
    //calling as_bytes inline because it returns &[u8] which is a temporary value
    stream.write(tcp_response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let thread_pool = ThreadPool::new(3);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("connection established!");
        thread_pool.execute(|| {
            connection_handler(stream);
        });
    }
}
