use std::{
    io::{self, Read, Write},
    net::TcpListener,
};

enum Path {
    Index,
    Echo,
    NotFound,
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        match value {
            "" => Self::Index,
            "echo" => Self::Echo,
            _ => Self::NotFound,
        }
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    loop {
        let (mut socket, _) = listener.accept().unwrap();

        let mut buf = [0; 1024];

        match socket.read(&mut buf) {
            Ok(_) => {
                let request = String::from_utf8_lossy(&buf);

                match extract_path(&request) {
                    Some(path) => {
                        let children: Vec<&str> = path.split('/').collect();

                        let mut res = String::new();
                        for child in children.clone() {
                            if children.len() == 1 && path == "/" {
                                res = "HTTP/1.1 200 OK\r\n\r\n".to_string();
                                break;
                            }

                            if child == "echo" {
                                res = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n{}\r\n\r\n", children[2].len(), children[2]);
                            } else {
                                res = "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                            }
                        }
                        socket.write(res.as_bytes())?
                    }
                    None => socket.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?,
                }
            }
            Err(_e) => socket.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?,
        };
    }
}

fn extract_path(req: &str) -> Option<&str> {
    for (idx, line) in req.lines().enumerate() {
        if idx == 0 {
            let parts: Vec<&str> = line.split_whitespace().collect();

            return parts.get(1).copied();
        }
    }

    None
}
