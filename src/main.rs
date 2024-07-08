use std::sync::Mutex;
use std::time::Duration;
use std::{fs, io, thread};
use std::net::{TcpListener, TcpStream};

use web_server::request::{parse_request, Request};
use web_server::response::{write_response, HttpStatus, Response};
use web_server::thread_pool::ThreadPool;

static mut SHUTDOWN: Mutex<bool> = Mutex::new(false);

fn main() {
    let addr = "127.0.0.1:7878";
    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).expect("Cannot set non-blocking");
    println!("listened at {addr}");

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => pool.execute(|| handle_connection(stream)),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                wait_for_fd();
            },
            Err(e) => panic!("encountered IO error: {e}")
        }
        // let stream = stream.unwrap();
        // println!("connection established!");
        let shutdown = unsafe {
            *SHUTDOWN.lock().unwrap()
        };
        if shutdown {
            pool.shutdown();
            break;
        }
    }
}

fn wait_for_fd() {
    // println!("wait_for_fd");
    thread::sleep(Duration::from_millis(100));
}
    
fn handle_connection(mut stream: TcpStream) {
    let request = parse_request(&mut stream);
    
    // println!("{request:#?}");

    let response = process_request(&request);

    write_response(&mut stream, &response);
}

fn process_request(request: &Request) -> Response {
    let (status, page) = match request.path.as_str() {
        "/" | "/index" | "/index.html" => (HttpStatus::OK, "pages/index.html"),
        "/hello.html" => (HttpStatus::OK, "pages/hello.html"),
        "/sleep" => {
            thread::sleep(Duration::from_secs(5));
            (HttpStatus::OK, "pages/hello.html")
        },
        "/shutdown" => {
            unsafe {
                *SHUTDOWN.lock().unwrap() = true;
            };
            (HttpStatus::OK, "pages/shutdown.html")
        },
        _ => (HttpStatus::NOT_FOUND, "pages/404.html")
    };
    
    let content = fs::read_to_string(page).unwrap();
    let content_length_heder = format!("Content-Length: {}", content.len());

    Response {
        status,
        headers: vec![content_length_heder],
        content
    }
}

