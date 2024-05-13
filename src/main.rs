use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};
pub struct ThreadPool {
    _threads: Vec<Worker>,
    sender: std::sync::mpsc::Sender<Job>,
}
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<std::sync::mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();
            println!("Worker {} got a job; executing.", id);
            job();
        });
        Worker { thread }
    }
}
impl ThreadPool {
    pub fn new(size: u32) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut wokers = Vec::with_capacity(size as usize);
        for id in 0..size {
            wokers.push(Worker::new(id as usize, Arc::clone(&receiver)));
        }
        ThreadPool {
            _threads: wokers,
            sender,
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| handle_connection(stream));
        println!("Connection established!");
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buffer = BufReader::new(&mut stream);

    let http_request: Vec<_> = buffer
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // A simple way to get the first line of the request
    let first_line = match http_request.first() {
        Some(line) => line,
        None => {
            let response = response_404();
            stream.write_all(response.as_bytes()).unwrap();
            return;
        }
    };
    http_request.first().unwrap();
    match first_line.as_str() {
        "GET / HTTP/1.1" => {
            let response = responde_index();
            stream.write_all(response.as_bytes()).unwrap();
        }
        _ => {
            let response = response_404();
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

fn responde_index() -> String {
    let contents = fs::read_to_string("index.html").unwrap();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );
    response
}

fn response_404() -> String {
    let contents = fs::read_to_string("404.html").unwrap();
    let response = format!(
        "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\n\r\n{}",
        contents.len(),
        contents
    );
    response
}
