use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use crate::thrp::ThreadPool;

pub fn web_init(tx: std::sync::mpsc::Sender<u8>) {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    let pool = ThreadPool::new(4, tx);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // tx.send(String::from("p: 0")).unwrap();
        // tx.send(String::from("c: 128")).unwrap();
        pool.execute(|| {
            // send message!
            handle_connection(stream)
        });
    }
}

fn find_subsequence<T>(haystack: &[T], needle: &[T]) -> Option<usize>
where
    for<'a> &'a [T]: PartialEq,
{
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn get_val(arr: &[u8], start_idx: usize) -> u8 {
    let mut res = 0u8;
    let mut idx = start_idx;

    loop {
        if !(arr[idx] >= ('0' as u8) && arr[idx] <= ('9' as u8)) {
            break;
        }
        res = res * 10 + arr[idx] - ('0' as u8);
        idx += 1;
    }
    res
}

fn handle_connection(mut stream: TcpStream) -> (u8, u8) {
    let mut buffer = [0u8; 2048];
    let mut press_res = 0u8;
    let mut code_res = 0u8;
    stream.read(&mut buffer).unwrap();

    let req_get_idx = b"GET / HTTP/1.1\r\n";
    let req_get_sleep = b"GET /sleep HTTP/1.1\r\n";
    let req_post_val = b"POST /value HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(req_get_idx) {
        ("HTTP/1.1 200 OK", "static/index.html")
    } else if buffer.starts_with(req_get_sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "static/index.html")
    } else if buffer.starts_with(req_post_val) {
        // println!("==========================");
        // for v in buffer.iter() {
        //     print!("{}", *v as char);
        // }
        // println!("++++++++++++++++++++++++++");

        let press_val = match find_subsequence(&buffer, b"press") {
            Some(v) => get_val(&buffer, v + 7),
            None => 0u8,
        };
        let code_val = match find_subsequence(&buffer, b"params") {
            Some(v) => get_val(&buffer, v + 8),
            None => 0u8,
        };

        // println!("press: {}, code: {}", press_val, code_val);
        press_res = press_val;
        code_res = code_val;

        ("HTTP/1.1 200 OK", "static/index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "static/404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    (press_res, code_res)

    // if (press_res, code_res) == (0u8, 0u8) {
    //     None
    // } else {
    //     Some((press_res, code_res))
    // }
    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
