use std::io::{Read, Write};
use std::net::TcpStream;
use std::fmt;
pub const MINCHUNKLEN: usize = 8;
pub const CHUNKLEN: usize = 512;
trait RequestEncodable {
    fn encode(&self) -> String;
}

#[derive(Debug)]
enum ReqIntegrityErrorType {
    BadHeaders,
    BadBody,
    BadMethod,
}

#[derive(Debug)]
struct ReqIntegrityError {
    pub kind: ReqIntegrityErrorType,
    pub message: String,
}

impl fmt::Display for ReqIntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_args!("ReqIntegrityError: {:?}\nInfo: {}", self.kind, self.message))
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Protocol {
    HTTP0_9,
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

impl Protocol {
    pub fn connect(&self, addr: String) -> Option<TcpStream> { match self {
        Protocol::HTTP0_9 | Protocol::HTTP1_0 | Protocol::HTTP1_1 => { TcpStream::connect(addr).ok()} _ => None,
    }}
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
#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
    is_complete: bool,
}
impl Header { pub fn empty() -> Self { Self { name: String::new(), value: String::new(), is_complete: false,}}
pub fn new(name: String, value: String) -> Self { Self { name, value, is_complete: true,}}
pub fn complete(&mut self){self.is_complete = true;}
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
        match self.protocol { Protocol::HTTP0_9 => {return format!("{} {}\r\n\r\n", self.method, self.requesttarget);} _ => {
            req.push_str(format!("{} {} {}\r\n", self.method, self.requesttarget, self.protocol.encode()).as_str(),);
            for header in self.headers.iter() {
                req.push_str(header.encode().as_str());
            }
        }  
    } 

        if self.body.is_some() {
            let body = self.body.as_ref().unwrap();
        if req.ends_with("\r\n") {
            req = req.strip_suffix("\r\n").unwrap().to_string();
        }
        req.push_str("\r\n\r\n");
        req.push_str(body);
        }
        req
    }
}

impl Req {
    fn ensureint(&self, client: &Client) -> Result<(), ReqIntegrityError> {
        match self.protocol {
            Protocol::HTTP0_9 => {
                if self.method !="GET" && !client.tolerant {
                    return Err(ReqIntegrityError { kind: ReqIntegrityErrorType::BadMethod, message: format!("use GET not {} for HTTP/0.9", self.method), });
                }
                if self.headers.len() != 0 && !client.tolerant {
                    return Err(ReqIntegrityError { kind: ReqIntegrityErrorType::BadHeaders, message: format!("found {} header only use 0 header for HTTP/0.9", self.headers.len()), });
                }
                if self.body.is_some() {
                    return Err(ReqIntegrityError { kind: ReqIntegrityErrorType::BadBody, message: format!("found '{}', no request body allowed with HTTP/0.9", self.body.as_ref().unwrap()), });
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn send(&self, client: &Client) -> Result<Response, ReqIntegrityError> {
        let int = self.ensureint(client);
        if int.is_err() {
            return Err(int.unwrap_err());
        }

        match self.protocol {
            Protocol::HTTP0_9 => {
                let mut stream = client.connection.as_ref().unwrap();
                _ = stream.write(self.encode().as_bytes());
                let mut response = Response::new(Protocol::HTTP0_9);
                loop {
                    let mut chunk = [0; CHUNKLEN];
                    let bytesread = stream.read(&mut chunk).unwrap();
                    if bytesread == 0 {
                        break;
                    }
                    response.decode_body_chunk(&chunk);
                }
                response.rmzero();
                Ok(response)
            }
            Protocol::HTTP1_0 | Protocol::HTTP1_1 => {
                let mut stream = client.connection.as_ref().unwrap();
                _ = stream.write(self.encode().as_bytes());
                let mut responsedec = ResDec::new();
                loop {
                    let mut resp: [u8; 512] = [0; 512];
                    let bytesread = stream.read(&mut resp).unwrap();
                    if bytesread == 0 {
                        break;
                    }
                    responsedec.decode(&resp[..bytesread]);
                }
                Ok(responsedec.response)
            }
            _ => todo!(),
        }
    }
}

pub enum ResDecState { Protocol, Status, Reason, HeaderName, HeaderValue, Body,}
// I wont let you decode this bit.
// I tried to use AI to fix my errors here but it kept fixing the formatting so I thought it was funny to leave it how I wrote it.
// Good Luck
pub struct ResDec {state: ResDecState, response: Response,}
impl ResDec {pub fn new() -> Self{Self{state: ResDecState::Protocol,response: Response::default(),}}
             pub fn decode(&mut self, data: &[u8]) {let strdate = String::from_utf8(data.to_vec()).unwrap().trim_start_matches(' ').to_string();
            match self.state {ResDecState::Protocol => {assert!(strdate.len() > MINCHUNKLEN);
            let (protocol, remaining) = strdate.split_once(" ").unwrap();
        self.response.protocol = Some(match protocol {"HTTP/1.0" => Protocol::HTTP1_0, "HTTP/1.1" => Protocol::HTTP1_1, "HTTP/2.0" => Protocol::HTTP2_0, "HTTP/3.0" => Protocol::HTTP3_0, _ => panic!("protocol recieved {}", protocol),});
    self.state = ResDecState::Status;
self.decode(remaining.as_bytes());}

ResDecState::Status => {if strdate.len() < 5 {return;}
let (status,remaining) = strdate.split_once(" ").unwrap();
self.response.status_code = Some(status.to_string().parse::<u32>().unwrap());
self.state = ResDecState::Reason;
self.decode(remaining.as_bytes());}
ResDecState::Reason => {
    if strdate.len() == 0 { return; }
    if strdate.starts_with("\r\n") {
        self.state = ResDecState::HeaderName;
        let remaining = strdate.strip_prefix("\r\n").unwrap();
        return self.decode (remaining.as_bytes());
    }
    if strdate.contains("\r\n"){let(reason,remaining) = strdate.split_once("\r\n").unwrap();
match &self.response.reason{Some(curr_reason) => self.response.reason = Some(curr_reason.to_owned() + reason),
None => {self.response.reason = Some(reason.to_string());}}
self.state = ResDecState::HeaderName;
self.decode(remaining.as_bytes());}}
ResDecState::HeaderName => {if strdate.len() == 0{return;} if strdate.starts_with(":") {let remaining=strdate.strip_prefix(":").unwrap().trim_start();
self.state = ResDecState::HeaderValue;
return self.decode(remaining.as_bytes());}
if strdate.starts_with("\r\n"){let remaining=strdate.strip_prefix("\r\n").unwrap();
self.state = ResDecState::Body;
return self.decode(remaining.as_bytes());}
if strdate.contains(":"){let (name_data,reamining)=strdate.split_once(":").unwrap();
match self.response.headers.last_mut(){
    Some(previous)=>{
        if !previous.is_complete {
            previous.name += name_data;
        } else {let mut newheader = Header::empty();
        newheader.name += name_data;
    self.response.headers.push(newheader);}
    } None => {let mut newheader = Header::empty();
    newheader.name += name_data;
self.response.headers.push(newheader);}
} self.state = ResDecState::HeaderValue;
return self.decode(reamining.as_bytes());
} else { match self.response.headers.last_mut() {Some(previous) => {if !previous.is_complete {previous.name += strdate.as_str();} else {let mut newheader = Header::empty();
newheader.name = strdate;
self.response.headers.push(newheader);}} None => { let mut newheader = Header::empty();
newheader.name = strdate;
self.response.headers.push(newheader);}
}}}
ResDecState::HeaderValue => {
    if strdate.len() == 0 {return;}
    if strdate.starts_with("\r\n") {let remaining = strdate.strip_prefix("\r\n").unwrap();
let last = self.response.headers.last_mut().unwrap();
last.complete();
self.state = ResDecState::HeaderName;
return self.decode(remaining.as_bytes());}
if strdate.contains("\r\n"){let (value_data, remaining) = strdate.split_once("\r\n").unwrap();
let prev = self.response.headers.last_mut().unwrap();
prev.value += value_data;
prev.complete();
self.state = ResDecState::HeaderName;
return self.decode(remaining.as_bytes());} else{
    let prev = self.response.headers.last_mut().unwrap();
    prev.value += strdate.as_str();
}
}
ResDecState::Body => {if strdate.len() == 0{return;}
match &self.response.body {
    Some(body) => self.response.body = Some(body.to_owned() + strdate.as_str()),
    None => self.response.body = Some(strdate),
}}
}}}


#[derive(Default, Debug)]
pub struct Response {
    pub protocol: Option<Protocol>, pub status_code: Option<u32>, pub reason: Option<String>, pub headers: Vec<Header>, pub body: Option<String>,
}
impl Response {
    fn new(protocol: Protocol) -> Self{ Self { protocol: Some(protocol), ..Default::default() }}
    fn decode_body_chunk(&mut self, chunk: &[u8]) {
        let mut body = self.body.clone().unwrap_or(String::new());
        body.push_str(str::from_utf8(chunk).unwrap());
        if body.ends_with("\r\n\r\n") { body = body.strip_suffix("\r\n\r\n").unwrap().to_string();}
        self.body = Some(body);
    }
    fn rmzero (&mut self) {match &self.body{Some(body) =>{self.body=Some(body.trim_end_matches("\0").to_string());}None => return,}}
}

#[derive(Default)]
pub struct Client { addr: Option<String>, connection: Option<TcpStream>, preferredprot: Option<Protocol>, tolerant: bool}
impl Client {
    pub fn new(prefers: Protocol, permissive: bool) -> Self{Self{preferredprot: Some(prefers),tolerant: permissive,..Default::default()}}
    pub fn connect_to(&mut self,addr: String) {
        self.addr = Some(addr.clone());
        match &self.preferredprot {Some(proto)=>{self.connection = proto.connect(addr);} None=>{
            self.preferredprot = Some(Protocol::HTTP1_1);
            self.connection = self.preferredprot.as_ref().unwrap().connect(addr);
        }}
    }

    pub fn send_request(&self, req: Req) -> Option<Response> {match req.send(self) {Ok(resp) => Some(resp), Err(e) => {eprintln!("{}",e); None
    }}
    } }