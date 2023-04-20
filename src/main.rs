use std::{
    borrow::BorrowMut,
    env::var,
    fmt::Error,
    fs::{self, File},
    io::{prelude::*, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
};

enum Content {
    Favicon(String, Vec<u8>),
    Index(String, String),
}

fn get_file_extension(path: &str) -> &str {
    // TODO: Handle possible query params
    let ext = path.split(".").last().unwrap();

    ext
}

fn get_file_content_type(extension: &str) -> &str {
    match extension {
        "ico" => "image/x-icon",
        "css" => "text/css",
        "js" => "application/javascript",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        // TODO: Handle more types correctly
        _ => "text/html",
    }
}

fn get_dir_content(path: String) -> Vec<String> {
    let mut _f: Vec<String> = Vec::new();

    for entry in fs::read_dir(path).unwrap() {
        let path: PathBuf = entry.unwrap().path().clone();
        let is_dir: bool = path.is_dir();

        if is_dir {
            let mut _c: Vec<String> = get_dir_content(path.to_str().unwrap().to_string());

            _f.append(&mut _c);
        } else {
            _f.push(path.to_str().unwrap().to_string())
        }
    }

    _f
}

fn get_content(req: Vec<String>) -> Result<Content, Error> {
    let req_parts: Vec<&str> = req[0].split(" ").collect();

    // Only allow GET requests
    if req_parts[0] != "GET" {
        return Err(Error);
    }

    let base_path = String::from("content");
    let file_path = req_parts[1];
    let full_path = format!("{}{}", base_path, file_path);

    let extension = get_file_extension(file_path);

    let content = match extension {
        // TODO: improve
        "ico" | "css" | "js" | "svg" | "png" | "jpg" | "jpeg" => {
            // Handle binary
            let file = File::open(full_path).unwrap();
            let mut reader = BufReader::new(file);
            let mut buffer = Vec::new();

            match reader.read_to_end(&mut buffer) {
                Err(_) => return Err(Error),
                _ => (),
            }

            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                get_file_content_type(extension),
                buffer.len()
            );

            Content::Favicon(headers, buffer)
        }
        "html" => {
            let content = fs::read_to_string(format!("{}.{}", full_path, "html"));

            match content {
                Ok(result) => {
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                        result.len()
                    );

                    Content::Index(headers, result)
                }
                Err(_) => return Err(Error),
            }
        }
        "txt" => {
            let sitemap = get_dir_content(base_path);
            let sitemap = sitemap.into_iter().filter(|f| {
                if f.contains("html") {
                    return true;
                }

                false
            });

            let content = sitemap
                .collect::<Vec<String>>()
                .join("\r\n")
                .clone()
                .replace("content/", "");

            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                content.len()
            );

            Content::Index(headers, content)
        }
        _ => {
            let content = fs::read_to_string(format!("{}index.html", full_path));

            match content {
                Ok(result) => {
                    let headers = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                        result.len()
                    );

                    Content::Index(headers, result)
                }
                Err(_) => return Err(Error),
            }
        }
    };

    Ok(content)
}

fn handle_error(mut stream: &TcpStream) {
    let response = "HTTP/1.1 200 OK\r\n\r\n";

    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    if http_request.len() == 0 {
        return handle_error(&stream);
    }

    let content = get_content(http_request);

    match content {
        Ok(Content::Favicon(header, buffer)) => {
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(buffer.as_slice()).unwrap();
            stream.flush().unwrap();
        }
        Ok(Content::Index(header, content)) => {
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(content.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        Err(_) => {
            let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

fn main() {
    println!("ENVS {:?}", var("TEST"));

    let addrs = [SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], 8080))];

    let listener = TcpListener::bind(&addrs[..]).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream)
    }
}
