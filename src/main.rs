mod response;

use std::io;
use std::sync::Arc;

use itertools::Itertools;
use response::HttpResponse;
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
        let (socket, _) = listener.accept().await?;
        let directry = directry.clone();
        tokio::spawn(handle_request(socket, directry));
    }
}

async fn handle_request(mut socket: tokio::net::TcpStream, directry: Arc<Mutex<String>>) {
    let mut buf = [0; 1024];

    let _ = match socket.read(&mut buf).await {
        Ok(_) => {
            let request = String::from_utf8_lossy(&buf);

            match extract_method(&request) {
                Method::Get => match extract_path(&request) {
                    Some(path) => {
                        let children: Vec<&str> = path.split('/').collect();

                        let res = if path == "/" {
                            HttpResponse::Ok {
                                content_type: "text/plain".to_string(),
                                body: "".to_string(),
                            }
                        } else {
                            match Path::from(children[1]) {
                                Path::Echo => {
                                    let content = children.iter().skip(2).join("/");
                                    HttpResponse::Ok {
                                        content_type: "text/plain".to_string(),
                                        body: content,
                                    }
                                }
                                Path::UserAgent => {
                                    let user_agent_txt = extract_user_agent(&request).unwrap();
                                    HttpResponse::Ok {
                                        content_type: "text/plain".to_string(),
                                        body: user_agent_txt.to_string(),
                                    }
                                }
                                Path::Files => {
                                    let dir = directry.lock().await;

                                    match fs::File::open(format!("{}/{}", dir, children[2])).await {
                                        Ok(mut file_name) => {
                                            let mut contents = vec![];
                                            file_name.read_to_end(&mut contents).await.unwrap();

                                            HttpResponse::Ok {
                                                content_type: "application/octet-stream"
                                                    .to_string(),
                                                body: String::from_utf8(contents).unwrap(),
                                            }
                                        }
                                        Err(_e) => HttpResponse::NotFound,
                                    }
                                }
                                Path::NotFound => HttpResponse::NotFound,
                            }
                        };

                        let res_txt = res.to_http_string();
                        socket.write(res_txt.as_bytes()).await
                    }
                    None => {
                        let res = HttpResponse::NotFound.to_http_string();
                        socket.write(res.as_bytes()).await
                    }
                },
                Method::Post { body } => match extract_path(&request) {
                    Some(path) => {
                        let children: Vec<&str> = path.split('/').collect();

                        let res = if path == "/" {
                            HttpResponse::Ok {
                                content_type: "text/plain".to_string(),
                                body: "".to_string(),
                            }
                        } else {
                            match Path::from(children[1]) {
                                Path::Echo => HttpResponse::Ok {
                                    content_type: "text/plain".to_string(),
                                    body: "".to_string(),
                                },
                                Path::UserAgent => HttpResponse::Ok {
                                    content_type: "text/plain".to_string(),
                                    body: "".to_string(),
                                },
                                Path::Files => {
                                    let dir = directry.lock().await;

                                    match fs::File::create(format!("{}/{}", dir, children[2])).await
                                    {
                                        Ok(mut file_name) => {
                                            file_name.write_all(body.as_bytes()).await.unwrap();
                                            HttpResponse::Created
                                        }
                                        Err(_e) => HttpResponse::NotFound,
                                    }
                                }
                                Path::NotFound => HttpResponse::NotFound,
                            }
                        };

                        let res_txt = res.to_http_string();
                        socket.write(res_txt.as_bytes()).await
                    }
                    None => {
                        let res = HttpResponse::NotFound.to_http_string();
                        socket.write(res.as_bytes()).await
                    }
                },
                Method::Unknown => {
                    let res = HttpResponse::NotFound.to_http_string();
                    socket.write(res.as_bytes()).await
                }
            }
        }
        Err(_e) => {
            let res = HttpResponse::NotFound.to_http_string();
            socket.write(res.as_bytes()).await
        }
    };
}

fn extract_method(req: &str) -> Method {
    let lines: Vec<&str> = req.lines().collect();
    if lines.is_empty() {
        return Method::Unknown;
    }

    if lines[0].starts_with("GET") {
        Method::Get
    } else if lines[0].starts_with("POST") {
        Method::Post {
            body: lines.last().unwrap().trim_end_matches('\x00').to_string(),
        }
    } else {
        Method::Unknown
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
