use std::{
    io::{self, Read, Write},
    net::TcpListener,
};

use itertools::Itertools;

enum Path {
    Echo,
    UserAgent,
    NotFound,
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        match value {
            "echo" => Self::Echo,
            "user-agent" => Self::UserAgent,
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

                        let res = if path == "/" {
                            "HTTP/1.1 200 OK\r\n\r\n".to_string()
                        } else {
                           match Path::from(children[1]) {
                                Path::Echo => {
                                    let content = children.iter().skip(2).join("/");
                                    format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", 
                                        content.len(), 
                                        content
                                    )
                                },
                                Path::UserAgent => {
                                    let user_agent_txt = extract_user_agent(&request).unwrap();
                                    format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", 
                                        user_agent_txt.len(), 
                                        user_agent_txt
                                    )
                                }
                                Path::NotFound => {
                                    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                                }
                            }
                            // if children[1] == "echo" {

                            // } else {
                            //     "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                            // }
                        };

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

fn extract_user_agent(req: &str) -> Option<&str> {
    for (_idx, line) in req.lines().enumerate() {
        if line.starts_with("User-Agent") {
            let parts: Vec<&str> = line.split_whitespace().collect();

            return parts.get(1).copied();
        }
    }

    None
}
