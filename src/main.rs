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

                println!("{:?}", request);

                match extract_path(&request) {
                    Some(path) => {
                        let paths: Vec<&str> = path.split('/').collect();

                        if paths[1] == "echo" {
                            let res = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n{}\r\n\r\n", paths[2].len(), paths[2]);
                            socket.write(res.as_bytes())?
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
