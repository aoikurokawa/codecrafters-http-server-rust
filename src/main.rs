use std::{
    io::{self, Read, Write},
    net::TcpListener,
};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    loop {
        let (mut socket, _) = listener.accept().unwrap();

        let mut buf = [0; 1024];
        socket.read(&mut buf)?;
        socket.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
        // for stream in listener.incoming() {
        //     match stream {
        //         Ok(_stream) => {
        //             println!("accepted new connection");
        //         }
        //         Err(e) => {
        //             println!("error: {}", e);
        //         }
        //     }
        // }
    }
}
