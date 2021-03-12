use crate::http::{ParseError, Request, Response, StatusCode};
use std::convert::TryFrom;
use std::io::Read;
use std::net::TcpListener;

pub trait Handler {
    fn handle_request(&mut self, request: &Request) -> Response;

    fn handle_bad_request(&mut self, e: &ParseError) -> Response {
        println!("Failed to parse request: {}", e);
        Response::new(StatusCode::BadRequest, None)
    }
}

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub fn run<T>(&self, mut handler: T)
    where
        T: Handler,
    {
        let listener = TcpListener::bind(&self.addr).unwrap();
        println!("Listening on {}", self.addr);

        loop {
            if self.listen(&listener, &mut handler).is_err() {
                continue;
            }
        }
    }

    fn listen<T>(&self, listener: &TcpListener, handler: &mut T) -> Result<(), ()>
    where
        T: Handler,
    {
        let (mut stream, _) = match listener.accept() {
            Ok(res) => res,
            Err(e) => {
                println!("Failed to establish a connection: {}", e);
                return Err(());
            }
        };

        let mut buffer = [0; 1024];

        if let Err(error) = stream.read(&mut buffer) {
            println!("Failed to read from connection: {}", error);
        }

        let response = match Request::try_from(&buffer[..]) {
            Ok(req) => handler.handle_request(&req),
            Err(e) => handler.handle_bad_request(&e),
        };

        if let Err(e) = response.send(&mut stream) {
            println!("Failed to send response: {}", e);
        }

        Ok(())
    }
}
