use std::error::Error;
use std::process;
use std::io::prelude::*;
use std::io::Cursor;
use std::str;
use std::env;
use std::net::TcpListener;
use std::net::TcpStream;
use std::ops::RangeInclusive;
use rodio::{Decoder, OutputStream, Sink};
use include_dir::{include_dir, Dir};
use clap::Parser;
use sourv::ThreadPool;

static PROJECT_DIR: Dir = include_dir!("./assets");

// TODOs
// - error handling
// - split out cli handling from main.rs

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
/// Play a sound by GETting a web URL, provides endpoints /health and /stapler
pub struct Args {
    /// Port to listen on
    #[clap(short, long, default_value_t = 7878, parse(try_from_str=port_in_range))]
    port: usize,

    /// Sound volume (from 0.0 - 2.0) 
    #[clap(short, long, default_value_t = 1.0, parse(try_from_str=volume_in_range))]
    volume: f32,
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;
fn port_in_range(s: &str) -> Result<usize, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{}` isn't a port number", s))?;
    if PORT_RANGE.contains(&port) {
        Ok(port)
    } else {
        Err(format!(
            "Port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

const VOLUME_RANGE: RangeInclusive<f32> = 0.0..=2.0;
fn volume_in_range(s: &str) -> Result<f32, String> {
    let volume: f32 = s
        .parse()
        .map_err(|_| format!("`{}` isn't a float", s))?;
    if VOLUME_RANGE.contains(&volume) {
        Ok(volume)
    } else {
        Err(format!(
            "Volume not in range {:.1}-{:.1}",
            VOLUME_RANGE.start(),
            VOLUME_RANGE.end()
        ))
    }
}

pub struct Config {
    pub port: usize,
    pub volume: f32,
}

impl Config {
    pub fn new(args: &Args) -> Result<Config, &'static str> {
        let mut port = args.port;
        let mut volume = args.volume;

        if ! env::var("SOURV_PORT").is_err() {
            port = port_in_range(&env::var("SOURV_PORT").unwrap()).unwrap();
        }
        if ! env::var("SOURV_VOLUME").is_err() {
            volume = volume_in_range(&env::var("SOURV_VOLUME").unwrap()).unwrap();
        }

        Ok(Config { 
            port, 
            volume,
         })
    }
}

fn main() {
    let args = Args::parse();
    let config = Config::new(&args).unwrap();

    if let Err(e) = run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).unwrap();    // TODO error handling
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
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(source);
    sink.set_volume(2.0);
    sink.sleep_until_end();
}