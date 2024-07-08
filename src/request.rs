use std::{
    io::{BufRead, BufReader, ErrorKind::WouldBlock}, 
    net::TcpStream
};
use regex::Regex;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub version: String
}

pub fn parse_request(stream: &mut TcpStream) -> Request {
    let reader = BufReader::new(stream);

    let request: Vec<String> = reader.lines()
        .map(|line| {
            match line {
                Ok(line) => Some(line),
                Err(ref e) if e.kind() == WouldBlock => {
                    println!("read request block!");
                    None
                },
                Err(e) => panic!("read request encountered error: {e}")
            }
        })
        .filter(|line| line.is_some())
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // println!("Request: {request:#?}");

    let line = &request[0];
    let re = Regex::new(r"(\w+) ([^ ]+) (\w+)/(\d+.\d+)").unwrap();
    let cap = re.captures(line).unwrap();
    let (_, [method, path, protocol, version]) = cap.extract();

    Request {
        method: method.to_string(),
        path: path.to_string(),
        protocol: protocol.to_string(),
        version: version.to_string()
    }
}