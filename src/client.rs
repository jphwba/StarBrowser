use std::io::{Read, Write, BufReader};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::fmt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use rustls::RootCertStore;
use rustls_pki_types::ServerName;
use trust_dns_resolver::Resolver;
use rustls_native_certs;
use rustls_pemfile;

trait Connection: Read + Write + Send {}
impl<T: Read + Write + Send> Connection for T {}
pub const MINCHUNKLEN: usize = 8;
pub const CHUNKLEN: usize = 512;
trait RequestEncodable {
    fn encode(&self) -> String;
}

#[derive(Debug, Clone, PartialEq)]
pub enum HTMLTag {
    A, Abbr, Address, Area, Article, Aside, Audio, B, Base, Bdi, Bdo, Blockquote,
    Body, Br, Button, Canvas, Caption, Cite, Code, Col, Colgroup, Data, Datalist,
    Dd, Del, Details, Dfn, Dialog, Div, Dl, Dt, Em, Embed, Fieldset, Figcaption,
    Figure, Footer, Form, H1, H2, H3, H4, H5, H6, Head, Header, Hgroup, Hr, Html,
    I, Iframe, Img, Input, Ins, Kbd, Label, Legend, Li, Link, Main, Map, Mark,
    Meta, Meter, Nav, Noscript, Object, Ol, Optgroup, Option, Output, P, Param,
    Picture, Pre, Progress, Q, Rp, Rt, Ruby, S, Samp, Script, Search, Section,
    Select, Slot, Small, Source, Span, Strong, Style, Sub, Summary, Sup, Svg,
    Table, Tbody, Td, Template, Textarea, Tfoot, Th, Thead, Time, Title, Tr,
    Track, U, Ul, Var, Video, Wbr, Custom(String),
}
#[derive(Debug, Clone)]
pub struct HTMLAttribute {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HTMLNode {
    pub tag: HTMLTag,
    pub attributes: Vec<HTMLAttribute>,
    pub children: Vec<HTMLNode>,
    pub text_content: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HTMLDocument {
    pub doctype: Option<String>,
    pub root: HTMLNode,
}

// Lowk just used AI for this HTML tag implementation im not typing all of them out manually
impl HTMLTag {
    pub fn from_string(s: &str) -> Self {
        match s.to_uppercase().as_str(){
            "A" => HTMLTag::A,
            "ABBR" => HTMLTag::Abbr,
            "ADDRESS" => HTMLTag::Address,
            "AREA" => HTMLTag::Area,
            "ARTICLE" => HTMLTag::Article,
            "ASIDE" => HTMLTag::Aside,
            "AUDIO" => HTMLTag::Audio,
            "B" => HTMLTag::B,
            "BASE" => HTMLTag::Base,
            "BDI" => HTMLTag::Bdi,
            "BDO" => HTMLTag::Bdo,
            "BLOCKQUOTE" => HTMLTag::Blockquote,
            "BODY" => HTMLTag::Body,
            "BR" => HTMLTag::Br,
            "BUTTON" => HTMLTag::Button,
            "CANVAS" => HTMLTag::Canvas,
            "CAPTION" => HTMLTag::Caption,
            "CITE" => HTMLTag::Cite,
            "CODE" => HTMLTag::Code,
            "COL" => HTMLTag::Col,
            "COLGROUP" => HTMLTag::Colgroup,
            "DATA" => HTMLTag::Data,
            "DATALIST" => HTMLTag::Datalist,
            "DD" => HTMLTag::Dd,
            "DEL" => HTMLTag::Del,
            "DETAILS" => HTMLTag::Details,
            "DFN" => HTMLTag::Dfn,
            "DIALOG" => HTMLTag::Dialog,
            "DIV" => HTMLTag::Div,
            "DL" => HTMLTag::Dl,
            "DT" => HTMLTag::Dt,
            "EM" => HTMLTag::Em,
            "EMBED" => HTMLTag::Embed,
            "FIELDSET" => HTMLTag::Fieldset,
            "FIGCAPTION" => HTMLTag::Figcaption,
            "FIGURE" => HTMLTag::Figure,
            "FOOTER" => HTMLTag::Footer,
            "FORM" => HTMLTag::Form,
            "H1" => HTMLTag::H1,
            "H2" => HTMLTag::H2,
            "H3" => HTMLTag::H3,
            "H4" => HTMLTag::H4,
            "H5" => HTMLTag::H5,
            "H6" => HTMLTag::H6,
            "HEAD" => HTMLTag::Head,
            "HEADER" => HTMLTag::Header,
            "HGROUP" => HTMLTag::Hgroup,
            "HR" => HTMLTag::Hr,
            "HTML" => HTMLTag::Html,
            "I" => HTMLTag::I,
            "IFRAME" => HTMLTag::Iframe,
            "IMG" => HTMLTag::Img,
            "INPUT" => HTMLTag::Input,
            "INS" => HTMLTag::Ins,
            "KBD" => HTMLTag::Kbd,
            "LABEL" => HTMLTag::Label,
            "LEGEND" => HTMLTag::Legend,
            "LI" => HTMLTag::Li,
            "LINK" => HTMLTag::Link,
            "MAIN" => HTMLTag::Main,
            "MAP" => HTMLTag::Map,
            "MARK" => HTMLTag::Mark,
            "META" => HTMLTag::Meta,
            "METER" => HTMLTag::Meter,
            "NAV" => HTMLTag::Nav,
            "NOSCRIPT" => HTMLTag::Noscript,
            "OBJECT" => HTMLTag::Object,
            "OL" => HTMLTag::Ol,
            "OPTGROUP" => HTMLTag::Optgroup,
            "OPTION" => HTMLTag::Option,
            "OUTPUT" => HTMLTag::Output,
            "P" => HTMLTag::P,
            "PARAM" => HTMLTag::Param,
            "PICTURE" => HTMLTag::Picture,
            "PRE" => HTMLTag::Pre,
            "PROGRESS" => HTMLTag::Progress,
            "Q" => HTMLTag::Q,
            "RP" => HTMLTag::Rp,
            "RT" => HTMLTag::Rt,
            "RUBY" => HTMLTag::Ruby,
            "S" => HTMLTag::S,
            "SAMP" => HTMLTag::Samp,
            "SCRIPT" => HTMLTag::Script,
            "SEARCH" => HTMLTag::Search,
            "SECTION" => HTMLTag::Section,
            "SELECT" => HTMLTag::Select,
            "SLOT" => HTMLTag::Slot,
            "SMALL" => HTMLTag::Small,
            "SOURCE" => HTMLTag::Source,
            "SPAN" => HTMLTag::Span,
            "STRONG" => HTMLTag::Strong,
            "STYLE" => HTMLTag::Style,
            "SUB" => HTMLTag::Sub,
            "SUMMARY" => HTMLTag::Summary,
            "SUP" => HTMLTag::Sup,
            "SVG" => HTMLTag::Svg,
            "TABLE" => HTMLTag::Table,
            "TBODY" => HTMLTag::Tbody,
            "TD" => HTMLTag::Td,
            "TEMPLATE" => HTMLTag::Template,
            "TEXTAREA" => HTMLTag::Textarea,
            "TFOOT" => HTMLTag::Tfoot,
            "TH" => HTMLTag::Th,
            "THEAD" => HTMLTag::Thead,
            "TIME" => HTMLTag::Time,
            "TITLE" => HTMLTag::Title,
            "TR" => HTMLTag::Tr,
            "TRACK" => HTMLTag::Track,
            "U" => HTMLTag::U,
            "UL" => HTMLTag::Ul,
            "VAR" => HTMLTag::Var,
            "VIDEO" => HTMLTag::Video,
            "WBR" => HTMLTag::Wbr,
            _ => HTMLTag::Custom(s.to_string()),
        }
    }
}

impl HTMLNode {
    pub fn new(tag: HTMLTag) -> Self{Self { tag, attributes: Vec::new(), children: Vec::new(), text_content: None, }}
    pub fn add_attribute(&mut self, name: String, value: Option<String>) {
        self.attributes.push(HTMLAttribute {name, value});
    }
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
    pub fn connect(&self, addr: &str) -> Option<TcpStream> { match self {
        Protocol::HTTP0_9 | Protocol::HTTP1_0 | Protocol::HTTP1_1 => {
            let socket = addr.to_socket_addrs().ok()?.next()?;
            let stream = TcpStream::connect_timeout(&socket, Duration::from_secs(5)).ok()?;
            let _ = stream.set_nodelay(true);
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            Some(stream)
        } _ => None,
    }}

    pub fn connect_tls(&self, addr: &str, domain: &str) -> Option<rustls::StreamOwned<rustls::ClientConnection, TcpStream>> {
        use rustls::ClientConfig;
        use std::sync::Arc;
        let socket = addr.to_socket_addrs().ok()?.next()?;
        let stream = TcpStream::connect_timeout(&socket, Duration::from_secs(5)).ok()?;
        let mut root_store = rustls::RootCertStore::empty();
        if let Ok(certs) = rustls_native_certs::load_native_certs() {
            for cert in certs {
                let _ = root_store.add(cert);
            }
        }
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AllowAnyVerifier::new(root_store)))
            .with_no_client_auth();
        let server_name = ServerName::try_from(domain)
            .ok()?
            .to_owned();
        let conn = rustls::ClientConnection::new(Arc::new(config), server_name).ok()?;
        let tls_stream = rustls::StreamOwned::new(conn, stream);
        Some(tls_stream)
    }
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
        format!("{}: {}\r\n", self.name, self.value)
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
        match self.protocol {
            Protocol::HTTP0_9 => {
                return format!("{} {}\r\n\r\n", self.method, self.requesttarget);
            }
            _ => {
                req.push_str(
                    format!(
                        "{} {} {}\r\n",
                        self.method,
                        self.requesttarget,
                        self.protocol.encode()
                    )
                    .as_str(),
                );
                for header in self.headers.iter() {
                    req.push_str(header.encode().as_str());
                }
            }
        }

        req.push_str("\r\n");
        if let Some(body) = &self.body {
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

    fn send(&self, client: &mut Client) -> Result<Response, ReqIntegrityError> {
        let int = self.ensureint(client);
        if int.is_err() {
            return Err(int.unwrap_err());
        }

        match self.protocol {
            Protocol::HTTP0_9 => {
                let stream = match client.connection.as_mut() {
                    Some(c) => c,
                    None => return Err(ReqIntegrityError { kind: ReqIntegrityErrorType::BadHeaders, message: "No connection".to_string() }),
                };
                if stream.write(self.encode().as_bytes()).is_err() {
                    return Err(ReqIntegrityError { kind: ReqIntegrityErrorType::BadHeaders, message: "Write failed".to_string() });
                }
                let mut response = Response::new(Protocol::HTTP0_9);
                loop {
                    let mut chunk = [0; CHUNKLEN];
                    match stream.read(&mut chunk) {
                        Ok(0) => break,
                        Ok(bytesread) => response.decode_body_chunk(&chunk[..bytesread]),
                        Err(_) => break,
                    }
                }
                response.rmzero();
                Ok(response)
            }
            Protocol::HTTP1_0 | Protocol::HTTP1_1 => {
                let stream = match client.connection.as_mut() {
                    Some(c) => c,
                    None => return Err(ReqIntegrityError { kind: ReqIntegrityErrorType::BadHeaders, message: "No connection".to_string() }),
                };
                if stream.write(self.encode().as_bytes()).is_err() {
                    return Err(ReqIntegrityError { kind: ReqIntegrityErrorType::BadHeaders, message: "Write failed".to_string() });
                }
                let mut responsedec = ResDec::new();
                loop {
                    let mut resp: [u8; 512] = [0; 512];
                    match stream.read(&mut resp) {
                        Ok(0) => break,
                        Ok(bytesread) => responsedec.decode(&resp[..bytesread]),
                        Err(_) => break,
                    }
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

#[derive(Debug, Clone)]
struct DNSCacheEntry{
    addresses: Vec<SocketAddr>,
    expiry: u64,
}
#[derive(Debug, Default)]
pub struct DNSCache {
    cache: Arc<Mutex<HashMap<String, DNSCacheEntry>>>,
    ttl: u64,
}
impl DNSCache {
    pub fn new(ttl_seconds: u64) -> Self {Self{
        cache: Arc::new(Mutex::new(HashMap::new())),
        ttl: ttl_seconds,
    }}

    pub fn get(&self, host: &str) -> Option<Vec<SocketAddr>> {
        let cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get(host) {
            let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now < entry.expiry {
            return Some(entry.addresses.clone());
        }
        }
        None
    }
    pub fn insert(&self, host: String, addresses: Vec<SocketAddr>) {
        let mut cache = self.cache.lock().unwrap();
        let expiry = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + self.ttl;
    cache.insert(host, DNSCacheEntry {addresses, expiry});
    }
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}
#[derive(Debug)]
struct AllowAnyVerifier {
    root_store: RootCertStore,
}
impl AllowAnyVerifier {
    fn new(root_store: RootCertStore) -> Self {Self{root_store}}
}
impl rustls::client::danger::ServerCertVerifier for AllowAnyVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls_pki_types::CertificateDer<'_>,
        _intermediates: &[rustls_pki_types::CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _oscp_response: &[u8],
        _now: rustls_pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls_pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
#[derive(Default)]
pub struct Client {
    addr: Option<String>,
    connection: Option<Box<dyn Connection>>,
    preferredprot: Option<Protocol>,
    tolerant: bool,
    dns_cache: DNSCache,
    redirect_count: usize,
    max_redirect: usize,
}
impl Client {
    pub fn new(prefers: Protocol, permissive: bool) -> Self {Self { 
        preferredprot: Some(prefers),
        tolerant: permissive,
        dns_cache: DNSCache::new(300),
        redirect_count: 0,
        max_redirect: 5,
        ..Default::default()
    }}
    pub fn connect_to(&mut self, addr: String) {
        self.addr = Some(addr.clone());
        let (host, port, use_tls) = self.parse_url(&addr);
        match &self.preferredprot {
            Some(proto) => {
                self.connection = self.resolve_and_connect(&host, port, use_tls, proto);
            }
            None => {
                self.preferredprot = Some(Protocol::HTTP1_1);
                self.connection = self.resolve_and_connect(&host, port, use_tls, self.preferredprot.as_ref().unwrap());
            }
        }
    }

    fn parse_url(&self, addr: &str) -> (String, u16, bool) {
        let mut use_tls = false;
        let mut host = addr.to_string();
        let mut port = 80u16;

        if host.starts_with("https://") {
            use_tls = true;
            port = 443;
            host = host.strip_prefix("https://").unwrap().to_string();
        } else if host.starts_with("http://") {
            host = host.strip_prefix("http://").unwrap().to_string();
        }

        if let Some(idx) = host.find(':') {
            let (host_part, port_part) = host.split_at(idx);
            let host_str = host_part.to_string();
            if let Ok(p) = port_part[1..].parse::<u16>() {
                port = p;
            }
            (host_str, port, use_tls)
        } else if use_tls && port == 80 {
            port = 443;
            (host, port, use_tls)
        } else{
        (host, port, use_tls)
        }
    }

    fn resolve_and_connect(&self, host: &str, port: u16, use_tls: bool, proto: &Protocol) -> Option<Box<dyn Connection>> {
        let socket_addrs = if let Some(cached) = self.dns_cache.get(host) {
            cached
        } else {
            match self.resolve_dns(host, port) {
                Ok(addrs) =>  {
                    self.dns_cache.insert(host.to_string(), addrs.clone());
                    addrs
                }
                Err(_) => return None,
            }
        };

        for addr in socket_addrs {
            if use_tls && matches!(proto, Protocol::HTTP1_1 | Protocol::HTTP2_0) {
                if let Some(tls_stream) = self.connect_tls(&addr.to_string(), host) {
                    return Some(Box::new(tls_stream));
                }
            } else {
                if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
                    let _ = stream.set_nodelay(true);
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    return Some(Box::new(stream));
                }
            }
        }
        None
    }

    fn resolve_dns(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error>> {
        use std::net::ToSocketAddrs;
        let addr_str = format!("{}:{}", host, port);
        if let Ok(addrs) = addr_str.to_socket_addrs() {
            return Ok(addrs.collect());
        }

        let resolver = Resolver::from_system_conf()?;
        let response = resolver.lookup_ip(host)?;
        let addrs: Vec<SocketAddr> = response.iter()
            .map(|ip| SocketAddr::new(ip, port))
            .collect();
        if addrs.is_empty() {
            return Err("No addresses found".into());
        }
        Ok(addrs)
    }
    fn connect_tls(&self, addr: &str, domain: &str) -> Option<rustls::StreamOwned<rustls::ClientConnection, TcpStream>> {
        use rustls::ClientConfig;
        use std::sync::Arc;
        let socket = addr.to_socket_addrs().ok()?.next()?;
        let stream = TcpStream::connect_timeout(&socket, Duration::from_secs(5)).ok()?;
        let mut root_store = RootCertStore::empty();

        if let Ok(file) = File::open("ca-certificates.crt") {
            let mut reader = BufReader::new(file);
            let certs = rustls_pemfile::certs(&mut reader);
            for cert in certs.flatten() {
                let _ = root_store.add(cert);
            }
        }

        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AllowAnyVerifier::new(root_store)))
            .with_no_client_auth();
        let server_name = ServerName::try_from(domain)
        .ok()?
        .to_owned();
    let conn = rustls::ClientConnection::new(Arc::new(config), server_name).ok()?;
    let tls_stream = rustls::StreamOwned::new(conn, stream);
    Some(tls_stream)
    }
    pub fn send_request(&mut self, mut req: Req) -> Option<Response> {
        if self.redirect_count >= self.max_redirect {
            eprintln!("Max redirects reached");
            return None;
        }
        match req.send(self) {
            Ok (resp) => {
                if let Some(status) = resp.status_code {
                    if status >= 300 && status < 400 {
                        if let Some(location) = resp.headers.iter()
                            .find(|h| h.name.to_lowercase() == "location") {
                                let val = location.value.clone();
                                self.redirect_count += 1;
                                self.addr = Some(val.clone());
                                self.connect_to(val.clone());
                                req.requesttarget = self.extract_path(&val);
                                return self.send_request(req);
                            }
                    }
                }
                self.redirect_count = 0;
                Some(resp)
            }
            Err(e) => {
                eprintln!("{}", e);
                None
            }
        }
    }
    fn extract_path(&self, url: &str) -> String {
        if let Some(idx) = url.find(|c: char| c == '/' || c == '?') {
            url[idx..].to_string()
        } else {
            "/".to_string()
        }
    }
}