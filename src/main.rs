use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::fs;
use std::path::Path;

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

        // extract the path from the request
        let path = request.lines().next().and_then(
            |line| line.split_whitespace().nth(1)
        ).unwrap_or("/");

        // remove the leading slash and map to index.html if empty
        let file_path = if path == "/" {
            "public/index.html"
        } else {
            &path[1..]
        };

        // check if file exists and try to read it
        let (status_line, content) = if Path::new(file_path).exists() {
            match fs::read_to_string(file_path) {
                Ok(contents) => ("HTTP/1.1 200 OK", contents),
                Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR", "Error reading file".to_string())
            }
        } else if path == "/hello" {
            ("HTTP/1.1 200 OK", "Hello, Rustacean!".to_string())
        } else {
            ("HTTP/1.1 404 NOT FOUND", "Page not found".to_string())
        };

        let content_type = get_content_type(file_path);
        let response = format!(
            "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            status_line,
            content_type,
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

fn get_content_type(path: &str) -> &str {
    let extension = Path::new(path).extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    match extension {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        _ => "text/plain",
    }
}
