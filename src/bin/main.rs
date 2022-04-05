use std::error::Error;
use std::env;
use std::process;
use std::io::prelude::*;
use std::str;
use std::net::TcpListener;
use std::net::TcpStream;
use sourv::ThreadPool;
use std::io::Cursor;
use rodio::{Decoder, OutputStream, source::Source};
use include_dir::{include_dir, Dir};
use sourv::Config;

static PROJECT_DIR: Dir = include_dir!("./assets");

// TODOs
// - error handling

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).unwrap();
    let pool = ThreadPool::new(2);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| { 
            handle_connection(stream);
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    
    let health = b"GET /health HTTP/1.1\r\n";
    let stapler = b"GET /stapler HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(health) {
        ("HTTP/1.1 200 OK", "health.html")
    } else if buffer.starts_with(stapler) {
        stapling_sound();
        ("HTTP/1.1 200 OK", "")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = if filename.len() > 0 {
        str::from_utf8(PROJECT_DIR.get_file(filename).unwrap().contents()).unwrap()
    } else {
        ""
    };
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();    
}

fn stapling_sound() {
    let sound = PROJECT_DIR.get_file("StaplingSound.wav").unwrap();
    let cursor = Cursor::new(sound.contents());
    let source = Decoder::new(cursor).unwrap();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    stream_handle.play_raw(source.convert_samples()).unwrap();
    
    std::thread::sleep(std::time::Duration::from_secs(1));
}