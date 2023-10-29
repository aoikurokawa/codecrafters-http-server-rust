use std::io;
use std::sync::Arc;

use itertools::Itertools;
use tokio::fs;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
};

enum Path {
    Echo,
    UserAgent,
    Files,
    NotFound,
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        match value {
            "echo" => Self::Echo,
            "user-agent" => Self::UserAgent,
            "files" => Self::Files,
            _ => Self::NotFound,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut directry = Arc::new(Mutex::new(String::new()));
    for (index, arg) in args.iter().enumerate() {
        if arg == "--directory" && index + 1 < args.len() {
            directry = Arc::new(Mutex::new(args[index + 1].clone()));
        }
    }

    let listener = TcpListener::bind("127.0.0.1:4221").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        let directry = directry.clone();

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            let _ = match socket.read(&mut buf).await {
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
                                    }
                                    Path::UserAgent => {
                                        let user_agent_txt = extract_user_agent(&request).unwrap();
                                        format!(
                                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", 
                                            user_agent_txt.len(),
                                            user_agent_txt
                                        )
                                    }
                                    Path::Files => {
                                        let dir = directry.lock().await;

                                        match fs::File::open(format!("{}/{}", dir, children[2]))
                                            .await
                                        {
                                            Ok(mut file_name) => {
                                                let mut contents = vec![];
                                                file_name.read_to_end(&mut contents).await.unwrap();

                                                format!(
                                            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", 
                                            contents.len(),
                                            String::from_utf8(contents).unwrap()
                                        )
                                            }
                                            Err(_e) => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                                        }
                                    }
                                    Path::NotFound => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                                }
                            };

                            socket.write(res.as_bytes()).await
                        }
                        None => socket.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await,
                    }
                }
                Err(_e) => socket.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await,
            };
        });
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
