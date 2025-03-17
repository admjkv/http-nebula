use serde::Deserialize;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;

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
        Ok(content) => match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Error parsing nebula.toml: {}. Using default config.", e);
                NebulaConfig::default()
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
                    if let Err(e) = handle_connection(stream, &thread_config) {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream, config: &NebulaConfig) -> Result<(), std::io::Error> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;

    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    // convert the request bytes to a string for logging
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("Request: {}", request);

    // Use the parse_http_request function to extract method and path
    let (method, path) = parse_http_request(&buffer)
        .unwrap_or(("GET", "/"));
    
    println!("Method: {}, Path: {}", method, path);

    // remove the leading slash and map to default file if empty
    let file_path = if path == "/" {
        format!(
            "{}/{}",
            config.content.public_dir, config.content.default_file
        )
    } else {
        format!("{}/{}", config.content.public_dir, sanitize_path(&path))
    };

    // Inside handle_connection after parsing the request
    let (status_line, content, _) = if method == "GET" {
        if Path::new(&file_path).exists() {
            let content_type = get_content_type(&file_path);
            let is_binary =
                !content_type.starts_with("text/") && content_type != "application/javascript";

            if is_binary {
                match fs::read(&file_path) {
                    Ok(contents) => ("HTTP/1.1 200 OK", contents, true),
                    Err(_) => (
                        "HTTP/1.1 500 INTERNAL SERVER ERROR",
                        Vec::from("Error reading file"),
                        false,
                    ),
                }
            } else {
                match fs::read_to_string(&file_path) {
                    Ok(contents) => ("HTTP/1.1 200 OK", contents.into_bytes(), false),
                    Err(_) => (
                        "HTTP/1.1 500 INTERNAL SERVER ERROR",
                        Vec::from("Error reading file"),
                        false,
                    ),
                }
            }
        } else if path == "/hello" {
            ("HTTP/1.1 200 OK", Vec::from("Hello, Rustacean!"), false)
        } else {
            ("HTTP/1.1 404 NOT FOUND", Vec::from("Page not found"), false)
        }
    } else {
        // Handle non-GET methods
        ("HTTP/1.1 405 METHOD NOT ALLOWED", Vec::from("Method not allowed"), false)
    };

    let content_type = get_content_type(&file_path);
    let response = format!(
        "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        status_line,
        content_type,
        content.len(),
    );

    stream.write_all(response.as_bytes())?;
    stream.write_all(&content)?;

    Ok(())
}

fn parse_http_request(buffer: &[u8]) -> Option<(&str, &str)> {
    let request = std::str::from_utf8(buffer).ok()?;
    let request_line = request.lines().next()?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    
    if parts.len() >= 2 {
        Some((parts[0], parts[1])) // (method, path)
    } else {
        None
    }
}

fn sanitize_path(path: &str) -> String {
    let path = path.trim_start_matches('/');
    let path_components: Vec<&str> = path.split('/').collect();

    let safe_components: Vec<&str> = path_components
        .into_iter()
        .filter(|component| !component.is_empty() && *component != "." && *component != "..")
        .collect();

    safe_components.join("/")
}

fn get_content_type(path: &str) -> &str {
    let extension = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "xml" => "application/xml",
        "webp" => "image/webp",
        _ => "text/plain",
    }
}
