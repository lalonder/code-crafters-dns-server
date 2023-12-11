mod parser;

use std::net::UdpSocket;
use bytes::BytesMut;
use parser::{parse_flags, parse_header, parse_question};
use parser::bits::parse_header_flags;

#[derive(Debug, Copy, Clone)]
struct Header {
    id: [u8; 2],
    flags: [u8; 2],
    qdcount: [u8; 2],
    ancount: [u8; 2],
    nscount: [u8; 2],
    arcount: [u8; 2],
}

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

#[derive(Debug)]
struct Question {
    qname: Vec<u8>,
    qtype: [u8; 2],
    qclass: [u8; 2],
}

#[derive(Debug, Clone)]
struct Answer {
    aname: Vec<u8>,
    atype: [u8; 2],
    aclass: [u8; 2],
    ttl: [u8; 4],
    rdlenth: [u8; 2],
    rdata: Vec<u8>,
}

#[derive(Debug)]
struct Message {
    header: Header,
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

impl Header {
    fn from(buf: &[u8]) -> Self {
        Header {
            id: [buf[0], buf[1]],
            flags: parse_flags(&[buf[2], buf[3]]),
            qdcount: [buf[4], buf[5]],
            ancount: [buf[6], buf[7]],
            nscount: [buf[8], buf[9]],
            arcount: [buf[10], buf[11]],
        }
    }

    fn to_vec(self) -> Vec<u8> {
        [
            self.id, self.flags, self.qdcount, self.ancount, self.nscount, self.arcount
        ].concat()
    }
}

impl Flags {
    fn parse(buf: &[u8]) -> Self {
        let (
            _,
            (qr, opcode, aa, tc, rd, ra, z, rcode)
        ) = parse_header_flags(buf).unwrap();
        Flags { qr, opcode, aa, tc, rd, ra, z, rcode }
    }
    fn set_response_indicator(&mut self) {
        self.qr |= 0b1000_0000;
    }
}

impl Question {
    fn from(buf: &[u8]) -> Question {
        parse_question(buf).unwrap().1
    }

    fn to_vec(&self) -> Vec<u8> {
        [&self.qname[..], &self.qtype[..], &self.qclass[..]].concat()
    }
}

impl Answer {
    fn from(buf: &[u8]) -> Self {
        let mut aname: Vec<u8> = buf.iter().take_while(|&x| *x != 0u8).cloned().collect();
        aname.push(0u8);
        Answer {
            aname,
            atype: 1u16.to_be_bytes(),
            aclass: 1u16.to_be_bytes(),
            ttl: 255u32.to_be_bytes(),
            rdlenth: 4u16.to_be_bytes(),
            rdata: Vec::from("\x08\x08\x08\x08".as_bytes()),
        }
    }

    fn to_vec(self) -> Vec<u8> {
        [
            &self.aname[..],
            &self.atype[..],
            &self.aclass[..],
            &self.ttl[..],
            &self.rdlenth[..],
            &self.rdata[..],
        ].concat()
    }
}

impl Message {
    fn from(buf: &[u8]) -> Self {
        Message {
            header: parse_header(buf).unwrap().1,
            questions: vec![Question::from(&buf[12..])],
            answers: {
                vec![Answer::from(&buf[12..])]
            },
        }
    }

    fn response(&mut self) -> Vec<u8> {
        // self.header.flags.set_response_indicator();
        self.header.qdcount = (self.questions.len() as u16).to_be_bytes();
        self.header.ancount = (self.answers.len() as u16).to_be_bytes();
        [
            self.header.to_vec(),
            self.questions.iter()
                .map(|x| x.to_vec())
                .collect::<Vec<Vec<u8>>>().concat(),
            self.answers.iter().cloned()
                .map(|x| x.to_vec())
                .collect::<Vec<Vec<u8>>>().concat(),
        ].concat()
    }
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to socket.");
    let mut buf: BytesMut = BytesMut::zeroed(512);
    loop {
        match udp_socket.recv_from(&mut buf[..]) {
            Ok((_size, source)) => {
                let mut message = Message::from(&buf);
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
