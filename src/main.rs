use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use std::net::{TcpListener, TcpStream};
use std::env;

const DEFAULT_PORT: &str = "6969";
const OUT_FILE: &str = "data.csv";

fn main() {
    let args: Vec<String> = env::args().collect();

    let port = match argument_value("--port", &args) {
        Ok(v) => v,
        Err(_) => match argument_value("-p", &args) {
            Ok(v) => v,
            Err(_) => {
                println!("No port specified, using the default one.");
                DEFAULT_PORT.to_owned()
            }
        }
    };

    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(listener) => listener,
        Err(e) => {
            println!("Failed to initialize TcpListener, perhaps the port is wrong? '{}'\nError: '{}'", port, e);
            return;
        }
    };

    println!("Running on port '{}'", port);
    
    let mut output = vec![];
    let mut exit = false;

    for stream in listener.incoming() {
        if exit {
            break;
        }
        match stream {
            Ok(mut s) => handle_stream(&mut s, &mut output, &mut exit),
            Err(e) => println!("Warning: Invalid stream\nError: '{}'", e)
        }
    }

    let csv = output.iter().fold(String::new(), |acc, v| format!("{acc}\n{v}"));
    let mut file = match File::create(OUT_FILE) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to write data!\nError: '{}'", e);
            return;
        }
    };
    file.write_all(csv.as_bytes()).unwrap();
    println!("Successfully wrote data to {OUT_FILE}\nExiting...");
}

fn handle_stream(stream: &mut TcpStream, output: &mut Vec<String>, exit: &mut bool) {
    let buf_reader = BufReader::new(stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if let Some(payload) = http_request.get(0) {
        let payload = payload
            .strip_prefix("GET ").unwrap()
            .strip_suffix(" HTTP/1.1").unwrap();
        
        if payload == "END" {
            *exit = true;
            return;
        }
        
        let payload: Vec<_> = payload.split(';').collect();
        println!("Data: {:?}", payload);
        for p in payload {
            output.push(p.to_owned());
        }
    }
}

fn argument_value(arg: &str, args: &Vec<String>) -> Result<String, ()> {
    let a = arg.to_owned();
    if !args.contains(&a) {
        return Err(());
    }
    let index = args.iter().position(|p| *p == a).unwrap();
    if let Some(value) = args.get(index + 1) {
        return Ok(value.to_owned());
    }
    Err(())
}
