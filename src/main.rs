mod parser;

use std::net::UdpSocket;
use bytes::BytesMut;
use crate::parser::parse_message;

#[derive(Debug, Copy, Clone)]
pub struct Flags {
    qr: u8,
    opcode: u8,
    aa: u8,
    tc: u8,
    rd: u8,
    ra: u8,
    z: u8,
    rcode: u8
}

#[derive(Debug, Copy, Clone)]
pub struct Header {
    id: u16,
    flags: Flags,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

#[derive(Debug, Clone)]
pub struct Question {
    qname: Vec<u8>,
    qtype: u16,
    qclass: u16,
}

#[derive(Debug, Clone)]
pub struct Answer {
    aname: Vec<u8>,
    atype: u16,
    aclass: u16,
    ttl: u32,
    rdlenth: u16,
    rdata: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Message {
    header: Header,
    question: Question,
}

#[derive(Debug, Clone)]
struct Response {
    header: Header,
    question: Question,
    answer: Answer,
}

impl Flags {
    fn set_response_flag(&mut self) {
        self.qr |= 0b1000_0000;
    }

    fn to_be_bytes(&self) -> [u8; 2] {
        [
            self.qr + self.opcode + self.aa + self.tc + self.rd,
            self.ra + self.z + self.rcode,
        ]
    }
}

impl Header {
    fn as_vec(&self) -> Vec<u8> {
        [
            self.id.to_be_bytes(),
            self.flags.to_be_bytes(),
            self.qdcount.to_be_bytes(),
            self.ancount.to_be_bytes(),
            self.nscount.to_be_bytes(),
            self.arcount.to_be_bytes(),
        ].concat()
    }
}

impl Question {
    fn as_vec(&self) -> Vec<u8> {
        [
            &self.qname[..],
            &self.qtype.to_be_bytes(),
            &self.qclass.to_be_bytes(),
        ].concat()
    }
}

impl Answer {
    fn as_vec(&self) -> Vec<u8> {
        [
            &self.aname[..],
            &self.atype.to_be_bytes(),
            &self.aclass.to_be_bytes(),
            &self.ttl.to_be_bytes(),
            &self.rdlenth.to_be_bytes(),
            &self.rdata[..],
        ].concat()
    }
}

impl Response {
    fn as_vec(&self) -> Vec<u8> {
        [
            self.header.as_vec(),
            self.question.as_vec(),
            // self.answer.as_vec(),
        ].concat()
    }
}

fn create_response(message: &Message) -> Response {
    let mut message = message.clone();
    message.header.flags.set_response_flag();
    message.header.ancount = message.header.qdcount;
    Response {
        header: message.header,
        question: message.question.clone(),
        answer: Answer {
            aname: message.question.qname,
            atype: 1u16,
            aclass: 1u16,
            ttl: 255u32,
            rdlenth: 4u16,
            rdata: Vec::from("\x08\x08\x08\x08".as_bytes()),
        },
    }
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to socket.");
    let mut buf: BytesMut = BytesMut::zeroed(512);
    loop {
        match udp_socket.recv_from(&mut buf[..]) {
            Ok((_size, source)) => {
                let (_, message) = parse_message(&buf).unwrap();
                let response = create_response(&message);
                println!("{:?}", response);
                println!("{:?}", response.as_vec());
                udp_socket
                   .send_to(&response.as_vec(), source)
                   .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
