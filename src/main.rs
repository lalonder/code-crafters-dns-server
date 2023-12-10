use std::net::UdpSocket;
use bytes::BytesMut;
use nom::IResult;
use nom::bytes::complete::{take, take_while};

#[derive(Debug, Copy, Clone)]
struct DnsHeader {
    id: [u8; 2],
    flags: [u8; 2],
    qdcount: [u8; 2],
    ancount: [u8; 2],
    nscount: [u8; 2],
    arcount: [u8; 2],
}

#[derive(Debug)]
struct DnsQuestion {
    qname: Vec<u8>,
    qtype: [u8; 2],
    qclass: [u8; 2],
}

#[derive(Debug, Clone)]
struct DnsAnswer {
    aname: Vec<u8>,
    atype: [u8; 2],
    aclass: [u8; 2],
    ttl: [u8; 4],
    rdlenth: [u8; 2],
    rdata: Vec<u8>,
}

#[derive(Debug)]
struct DnsMessage {
    header: DnsHeader,
    // questions: Vec<DnsQuestion>,
    // answers: Vec<DnsAnswer>,
}

impl DnsHeader {
    fn from(buf: &BytesMut) -> Self {
        DnsHeader {
            id: [buf[0], buf[1]],
            flags: parse_flags(&[buf[2], buf[3]]),
            qdcount: [buf[4], buf[5]],
            ancount: [buf[6], buf[7]],
            nscount: [buf[8], buf[9]],
            arcount: [buf[10], buf[11]],
        }
    }

    fn set_response_indicator(&mut self) {
        self.flags[0] |= 0b1000_0000;
    }

    fn to_vec(&self) -> Vec<u8> {
        [
            self.id, self.flags, self.qdcount, self.ancount, self.nscount, self.arcount
        ].concat()
    }
}

impl DnsQuestion {
    fn parse(buf: &BytesMut) -> DnsQuestion {
        let (_, (qname, qtype, qclass)) = parse_dn(buf).expect("Couldn't parse domain name.");
        DnsQuestion {
            qname: Vec::from(qname),
            qtype,
            qclass,
        }
    }

    fn to_vec(&self) -> Vec<u8> {
        [&self.qname[..], &self.qtype[..], &self.qclass[..]].concat()
    }
}

impl DnsAnswer {
    fn new() -> Self {
        DnsAnswer {
            aname: Vec::new(),
            atype: 1u16.to_be_bytes(),
            aclass: 1u16.to_be_bytes(),
            ttl: 255u32.to_be_bytes(),
            rdlenth: 4u16.to_be_bytes(),
            rdata: Vec::new(),
        }
    }

    fn to_vec(&self) -> Vec<u8> {
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

impl DnsMessage {
    fn from(buf: &BytesMut) -> Self {
        DnsMessage {
            header: DnsHeader::from(buf),
            // questions: vec![DnsQuestion::parse(buf)],
            // answers: vec![DnsAnswer::new()],
        }
    }

    fn response(&mut self) -> Vec<u8> {
        let response: Vec<u8> = Vec::new();
        self.header.set_response_indicator();
        // self.header.ancount = (self.answers.len() as u16).to_be_bytes();
        response
    }

}

fn parse_flags(bytes: &[u8; 2]) -> [u8; 2] {
    let qr =     bytes[0] & 0b1000_0000;
    let opcode = bytes[0] & 0b0111_1000;
    let aa =     0b0000_0000u8;
    let tc =     0b0000_0000u8;
    let rd =     bytes[0] & 0b0000_0001;
    let ra =     bytes[1] & 0b1000_0000;
    let z =      0b0000_0000;
    let rcode =  if opcode == 0u8 { 0u8 } else { 4u8 };
    [qr+opcode+aa+tc+rd, ra+z+rcode]
}

fn parse_dn(buf: &[u8]) -> IResult<&[u8], (&[u8], [u8; 2], [u8; 2])> {
    let (names, _) = take_while(|x| x != "\x00".as_bytes()[0])(buf)?;
    let (qtype,_ ) = take(2usize)(buf)?;
    let (qclass, _) = take(2usize)(buf)?;
    Ok((buf, (names, [qtype[0], qtype[1]], [qclass[0], qclass[1]])))
}

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to socket.");
    let mut buf: BytesMut = BytesMut::with_capacity(512);
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_size, source)) => {
                let mut message = DnsMessage::from(&buf);
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
