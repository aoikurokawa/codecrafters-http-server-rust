use std::{
    io::{self, Read, Write},
    net::TcpListener,
};

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
                        if path.starts_with("/") {
                            let children: Vec<&str> = path.split("/").collect();
                            if children.len() <= 1 {
                                socket.write(b"HTTP/1.1 200 OK\r\n\r\n")?
                            } else {
                                socket.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?
                            }
                        } else {
                            socket.write(b"HTTP/1.1 404 Not Found\r\n\r\n")?
                        }
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
