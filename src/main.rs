mod parser;

use std::net::UdpSocket;
use bytes::BytesMut;
use parser::{parse_header, parse_question};

#[derive(Debug, Copy, Clone)]
struct Flags {
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
struct Header {
    id: u16,
    flags: Flags,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

#[derive(Debug)]
struct Question {
    qname: Vec<u8>,
    qtype: u16,
    qclass: u16,
}

#[derive(Debug, Clone)]
struct Answer {
    aname: Vec<u8>,
    atype: u16,
    aclass: u16,
    ttl: u32,
    rdlenth: u16,
    rdata: Vec<u8>,
}

#[derive(Debug)]
struct Message {
    header: Header,
    question: Question,
    // answers: Vec<Answer>,
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
    fn to_vec(&self) -> Vec<u8> {
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
    fn to_vec(&self) -> Vec<u8> {
        [
            &self.qname[..],
            &self.qtype.to_be_bytes(),
            &self.qclass.to_be_bytes(),
        ].concat()
    }
}

impl Answer {
    fn from(buf: &[u8]) -> Self {
        let mut aname: Vec<u8> = buf.iter().take_while(|&x| *x != 0u8).cloned().collect();
        aname.push(0u8);
        Answer {
            aname,
            atype: 1u16,
            aclass: 1u16,
            ttl: 255u32,
            rdlenth: 4u16,
            rdata: Vec::from("\x08\x08\x08\x08".as_bytes()),
        }
    }

    fn to_vec(self) -> Vec<u8> {
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

impl Message {
    fn response(&mut self) -> Vec<u8> {
        // self.header.flags.set_response_indicator();
        self.header.qdcount = (self.question.qname == Vec::new()) as u16;
        self.header.ancount = 0u16;
        [
            self.header.to_vec(),
            self.question.to_vec(),
        ].concat()
    }
}

fn create_message(buf: &[u8]) -> Message {
    Message {
        header: match parse_header(buf) {
            Ok((_, header)) => header,
            _ => panic!(),
        },
        question: match parse_question(buf) {
            Ok((_, question)) => question,
            _ => panic!(),
        }
    }
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to socket.");
    let mut buf: BytesMut = BytesMut::zeroed(512);
    loop {
        match udp_socket.recv_from(&mut buf[..]) {
            Ok((_size, source)) => {
                let mut message = create_message(&buf);
                let response = message.response();
                println!("{:?}", buf);
                udp_socket
                   .send_to(&response, source)
                   .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
