use std::io::{Read, Write};
use std::net::TcpStream;

trait RequestEncodable {
    fn encode(&self) -> String;
}

#[allow(dead_code)]
pub enum Protocol {
    HTTP0_9,
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

impl RequestEncodable for Protocol {
    fn encode(&self) -> String {
        String::from(match self {
            Self::HTTP0_9 => "HTTP/0.9",
            Self::HTTP1_0 => "HTTP/1.0",
            Self::HTTP1_1 => "HTTP/1.1",
            Self::HTTP2_0 => "HTTP/2.0",
            Self::HTTP3_0 => "HTTP/3.0",
        })
    }
}

pub struct Header {
    pub name: String,
    pub value: String,
}

impl RequestEncodable for Header {
    fn encode(&self) -> String {
        format!("{}: {}\n", self.name, self.value)
    }
}

pub struct Req {
    pub method: String,
    pub requesttarget: String,
    pub protocol: Protocol,
    pub headers: Vec<Header>,
    pub body: Option<String>,
}

impl RequestEncodable for Req {
    fn encode(&self) -> String {
        let mut req = String::new();
        req.push_str(
            format!(
                "{} {} {}r\n",
                self.method,
                self.requesttarget,
                self.protocol.encode()
            )
            .as_str(),
        );

        for header in self.headers.iter() {
            req.push_str(header.encode().as_str());
        }

        if self.body.is_some() {
            let body = self.body.as_ref().unwrap();
            req.push_str("\r\n");
            req.push_str(body.as_str());
        }

        if req.ends_with("\r\n") {
            req = req.strip_suffix("\r\n").unwrap().to_string();
        }
        req.push_str("\r\n\r\n");

        req
    }
}

pub struct Response {}

pub fn send_request(req: Req) -> Response {
    let mut stream = TcpStream::connect("google.com:80").unwrap();

    _ = stream.write(req.encode().as_bytes());

    loop {
        let mut resp: [u8; 512] = [0; 512];
        let bytes_read = stream.read(&mut resp).unwrap();

        if bytes_read == 0 {
            break;
        }
        println!("{}", str::from_utf8(&resp).unwrap());
    }
    Response {}
}