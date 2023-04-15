use std::{
    fmt::Error,
    fs::{self, File},
    io::{prelude::*, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
};

enum Content {
    Favicon(String, Vec<u8>),
    Index(String, String),
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

    let content = match file_path {
        "/favicon.ico" => {
            let file = File::open(full_path).unwrap();
            let mut reader = BufReader::new(file);
            let mut buffer = Vec::new();

            match reader.read_to_end(&mut buffer) {
                Err(_) => return Err(Error),
                _ => (),
            }

            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/x-icon\r\nContent-Length: {}\r\n\r\n",
                buffer.len()
            );

            Content::Favicon(headers, buffer)
        }
        "/" => {
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
        _ => {
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
    };

    Ok(content)
}

fn handle_error(mut stream: &TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

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
    let addrs = [SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], 8080))];

    let listener = TcpListener::bind(&addrs[..]).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream)
    }
}
