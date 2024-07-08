use std::{io::Write, net::TcpStream};

use crate::LINE_SEP;

pub struct Response {
    pub status: HttpStatus,
    pub headers: Vec<String>,
    pub content: String
}

pub fn write_response(stream: &mut TcpStream, response: &Response) {
    let HttpStatus {code, message} = response.status;
    let status_line = format!("HTTP/1.1 {code} {message}");

    stream.write_all(status_line.as_bytes()).unwrap();
    stream.write_all(LINE_SEP.as_bytes()).unwrap();

    for header in &response.headers {
        stream.write_all(header.as_bytes()).unwrap();
        stream.write_all(LINE_SEP.as_bytes()).unwrap();
    }
    
    stream.write_all(LINE_SEP.as_bytes()).unwrap();
    stream.write_all(response.content.as_bytes()).unwrap();
}

pub struct HttpStatus {
    code: u32, 
    message: &'static str
}

impl HttpStatus {
    pub const OK: HttpStatus = Self {code: 200, message: "Ok"};
    pub const NOT_FOUND: HttpStatus = Self {code: 404, message: "Not Found"};
}
