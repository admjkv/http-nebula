use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> std::io::Result<()> {
    // bind the tcp listener to localhost on port 7878
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server is listening on http://127.0.0.1:7878");

    // accept incoming connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each connection
                thread::spawn(|| {
                    handle_connection(stream);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    if let Ok(_) = stream.read(&mut buffer) {
        // convert the request bytes to a string
        let request = String::from_utf8_lossy(&buffer[..]);
        println!("Request: {}", request);

        // check the request line
        let (status_line, content) = if request.starts_with("GET /hello ") {
            ("HTTP/1.1 200 OK", "Hello, Rustacean!")
        } else if request.starts_with("GET / ") {
            ("HTTP/1.1 200 OK", "Welcome to the homepage!")
        } else {
            ("HTTP/1.1 404 NOT FOUND", "Page not found")
        };

        let response = format!(
            "{}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            content.len(),
            content
        );

        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Failed to write to stream: {}", e);
        }
    } else {
        eprintln!("Failed to read from stream");
    }
}
