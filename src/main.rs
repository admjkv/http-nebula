use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn main() -> std::io::Result<()> {
    // bind the tcp listener to localhost on port 7878
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server is listening on http://127.0.0.1:7878");

    // accept incoming connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // handle each connection
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    // read request into buffer
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            // log the request
            println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

            // create a simple http response
            let response = "HTTP/1.1 200 OK\r\nContent-Lenght: 13\r\n\r\nHello, World!";
            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Failed to write to stream: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to read from stream: {}", e);
        }
    }
}
