use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct NebulaConfig {
    server: ServerConfig,
    content: ContentConfig,
}

#[derive(Deserialize, Clone)]
struct ServerConfig {
    address: String,
    port: u16,
}

#[derive(Deserialize, Clone)]
struct ContentConfig {
    public_dir: String,
    default_file: String,
}

impl Default for NebulaConfig {
    fn default() -> Self {
        NebulaConfig {
            server: ServerConfig {
                address: "127.0.0.1".to_string(),
                port: 7878,
            },
            content: ContentConfig {
                public_dir: "public".to_string(),
                default_file: "index.html".to_string(),
            },
        }
    }
}

fn load_config() -> NebulaConfig {
    match fs::read_to_string("nebula.toml") {
        Ok(content) => {
            match toml::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Error parsing nebula.toml: {}. Using default config.", e);
                    NebulaConfig::default()
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to read nebula.toml: {}. Using default config.", e);
            NebulaConfig::default()
        }
    }
}

fn main() -> std::io::Result<()> {
    // Load configuration
    let config = load_config();
    
    // bind the tcp listener to configured address and port
    let listener_addr = format!("{}:{}", config.server.address, config.server.port);
    let listener = TcpListener::bind(&listener_addr)?;
    println!("Server is listening on http://{}", listener_addr);

    // accept incoming connections in a loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Clone config for the new thread
                let thread_config = config.clone();
                
                // Spawn a new thread for each connection
                thread::spawn(move || {
                    handle_connection(stream, &thread_config);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, config: &NebulaConfig) {
    let mut buffer = [0; 1024];
    if let Ok(_) = stream.read(&mut buffer) {
        // convert the request bytes to a string
        let request = String::from_utf8_lossy(&buffer[..]);
        println!("Request: {}", request);

        // extract the path from the request
        let path = request.lines().next().and_then(
            |line| line.split_whitespace().nth(1)
        ).unwrap_or("/");

        // remove the leading slash and map to default file if empty
        let file_path = if path == "/" {
            format!("{}/{}", config.content.public_dir, config.content.default_file)
        } else {
            format!("{}{}", config.content.public_dir, path)
        };

        // check if file exists and serve it
        let (status_line, content, is_binary) = if Path::new(&file_path).exists() {
            let content_type = get_content_type(&file_path);
            let is_binary = !content_type.starts_with("text/") && content_type != "application/javascript";
            
            if is_binary {
                match fs::read(&file_path) {
                    Ok(contents) => ("HTTP/1.1 200 OK", contents, true),
                    Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR", Vec::from("Error reading file"), false)
                }
            } else {
                match fs::read_to_string(&file_path) {
                    Ok(contents) => ("HTTP/1.1 200 OK", contents.into_bytes(), false),
                    Err(_) => ("HTTP/1.1 500 INTERNAL SERVER ERROR", Vec::from("Error reading file"), false)
                }
            }
        } else if path == "/hello" {
            ("HTTP/1.1 200 OK", Vec::from("Hello, Rustacean!"), false)
        } else {
            ("HTTP/1.1 404 NOT FOUND", Vec::from("Page not found"), false)
        };

        let content_type = get_content_type(&file_path);
        let response = format!(
            "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            status_line,
            content_type,
            content.len(),
        );

        if let Err(e) = stream.write_all(response.as_bytes()) {
            eprintln!("Failed to write response headers: {}", e);
            return;
        }
        
        if let Err(e) = stream.write_all(&content) {
            eprintln!("Failed to write response body: {}", e);
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
