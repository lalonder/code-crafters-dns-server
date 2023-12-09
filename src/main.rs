use std::net::UdpSocket;

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

#[derive(Debug)]
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
    question: DnsQuestion,
    answer: DnsAnswer,
}

impl DnsAnswer {
    fn from(buf: &[u8]) -> Self {
        DnsAnswer {
            aname: parse_domain_names(buf),
            atype: 1u16.to_be_bytes(),
            aclass: 1u16.to_be_bytes(),
            ttl: 42u32.to_be_bytes(),
            rdlenth: 4u16.to_be_bytes(),
            rdata: Vec::from("\x08\x08\x08\x08"),
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

impl DnsHeader {
    fn from(buffer: &[u8]) -> Self {
        DnsHeader {
            id: [buffer[0], buffer[1]],
            flags: parse_flags([buffer[2], buffer[3]]),
            qdcount: [buffer[4], buffer[5]],
            ancount: [buffer[6], buffer[7]],
            nscount: [buffer[8], buffer[9]],
            arcount: [buffer[10], buffer[11]],
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
    fn from(buf: &[u8]) -> Self {
        DnsQuestion {
            qname: parse_domain_names(buf),
            qtype: 1u16.to_be_bytes(),
            qclass: 1u16.to_be_bytes(),
        }
    }

    fn to_vec(&self) -> Vec<u8> {
        [
            &self.qname[..],
            &self.qtype[..],
            &self.qclass[..],
        ].concat()
    }
}

impl DnsMessage {
    fn from(buf: &[u8]) -> Self {
        DnsMessage {
            header: DnsHeader::from(buf),
            question: DnsQuestion::from(buf),
            answer: DnsAnswer::from(buf)
        }
    }

    fn response(&mut self) -> Vec<u8> {
        self.header.set_response_indicator();
        self.header.qdcount = 1u16.to_be_bytes();
        self.header.ancount = 1u16.to_be_bytes();
        [
            self.header.to_vec(),
            self.question.to_vec(),
            self.answer.to_vec(),
        ].concat()
    }
}

fn parse_flags(bytes: [u8; 2]) -> [u8; 2] {
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

fn parse_domain_names(buf: &[u8]) -> Vec<u8> {
    const START: usize = 13;
    buf[START..].iter().cloned().take_while(|x| *x != 0u8).collect()
}

fn main() {
    println!("Logs from your program will appear here!");

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to socket.");
    let mut buf = [0; 512];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((_size, source)) => {
                let mut message = DnsMessage::from(&buf);
                let response = message.response();
                println!("{:?}", message.header);
                println!("{:?}", response);
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
