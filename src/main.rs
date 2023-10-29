use std::io;
use std::sync::Arc;

use itertools::Itertools;
use tokio::fs;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
};

enum Method {
    Get,
    Post { body: String },
    Unknown,
}

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

                    match extract_method(&request) {
                        Method::Get => match extract_path(&request) {
                            Some(path) => {
                                let children: Vec<&str> = path.split('/').collect();

                                let res = if path == "/" {
                                    http_response(200, "OK", "", "")
                                } else {
                                    match Path::from(children[1]) {
                                        Path::Echo => {
                                            let content = children.iter().skip(2).join("/");
                                            http_response(200, "OK", "text/plain", &content)
                                        }
                                        Path::UserAgent => {
                                            let user_agent_txt =
                                                extract_user_agent(&request).unwrap();
                                            http_response(200, "OK", "text/plain", user_agent_txt)
                                        }
                                        Path::Files => {
                                            let dir = directry.lock().await;

                                            match fs::File::open(format!("{}/{}", dir, children[2]))
                                                .await
                                            {
                                                Ok(mut file_name) => {
                                                    let mut contents = vec![];
                                                    file_name
                                                        .read_to_end(&mut contents)
                                                        .await
                                                        .unwrap();

                                                    http_response(
                                                        200,
                                                        "OK",
                                                        "application/octet-stream",
                                                        &String::from_utf8(contents).unwrap(),
                                                    )
                                                }
                                                Err(_e) => http_response(404, "Not Found", "", ""),
                                            }
                                        }
                                        Path::NotFound => http_response(404, "Not Found", "", ""),
                                    }
                                };

                                socket.write(res.as_bytes()).await
                            }
                            None => {
                                let res = http_response(404, "Not Found", "", "");
                                socket.write(res.as_bytes()).await
                            }
                        },
                        Method::Post { body } => match extract_path(&request) {
                            Some(path) => {
                                let children: Vec<&str> = path.split('/').collect();

                                let res = if path == "/" {
                                    http_response(200, "OK", "text/plain", "")
                                } else {
                                    match Path::from(children[1]) {
                                        Path::Echo => http_response(200, "OK", "text/plain", ""),
                                        Path::UserAgent => {
                                            http_response(200, "OK", "text/plain", "")
                                        }
                                        Path::Files => {
                                            let dir = directry.lock().await;

                                            match fs::File::create(format!(
                                                "{}/{}",
                                                dir, children[2]
                                            ))
                                            .await
                                            {
                                                Ok(mut file_name) => {
                                                    file_name
                                                        .write_all(body.as_bytes())
                                                        .await
                                                        .unwrap();
                                                    http_response(201, "OK", "text/plain", "")
                                                }
                                                Err(_e) => http_response(
                                                    404,
                                                    "Not Found",
                                                    "text/plain",
                                                    "",
                                                ),
                                            }
                                        }
                                        Path::NotFound => {
                                            http_response(404, "Not Found", "text/plain", "")
                                        }
                                    }
                                };

                                socket.write(res.as_bytes()).await
                            }
                            None => {
                                let res = http_response(404, "Not Found", "text/plain", "");

                                socket.write(res.as_bytes()).await
                            }
                        },
                        Method::Unknown => {
                            let res = http_response(404, "Not Found", "text/plain", "");
                            socket.write(res.as_bytes()).await
                        }
                    }
                }
                Err(_e) => {
                    let res = http_response(404, "Not Found", "text/plain", "");
                    socket.write(res.as_bytes()).await
                }
            };
        });
    }
}

fn http_response(status_code: u16, status: &str, content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {}\r\n{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        status,
        content_type,
        body.len(),
        body
    )
}

fn extract_method(req: &str) -> Method {
    let lines = req.lines();

    for (idx, line) in lines.clone().enumerate() {
        if idx == 0 {
            if line.starts_with("GET") {
                return Method::Get;
            } else if line.starts_with("POST") {
                Method::Post {
                    body: String::new(),
                }
            } else {
                return Method::Unknown;
            };
        }
    }

    Method::Post {
        body: lines.last().unwrap().trim_end_matches('\x00').to_string(),
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
