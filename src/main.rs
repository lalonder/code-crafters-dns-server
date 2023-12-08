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
}

impl DnsAnswer {
    fn new() -> Self {
        DnsAnswer {
            aname: Vec::from("\x0ccodecrafters\x02io\x00"),
            atype: 1u16.to_be_bytes(),
            aclass: 1u16.to_be_bytes(),
            ttl: 42u32.to_be_bytes(),
            rdlenth: 4u16.to_be_bytes(),
            rdata: Vec::from("\x08\x08\x08\x08"),
        }
    }

    fn as_vec(&self) -> Vec<u8> {
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
            flags: [buffer[2], buffer[3]],
            qdcount: [buffer[4], buffer[5]],
            ancount: [buffer[6], buffer[7]],
            nscount: [buffer[8], buffer[9]],
            arcount: [buffer[10], buffer[11]],
        }
    }

    fn parse_flags(&mut self) {
        let qr =     self.flags[0] & 0b1000_0000;
        let opcode = self.flags[0] & 0b0111_1000;
        let aa =     0b0000_0000u8;
        let tc =     0b0000_0000u8;
        let rd =     self.flags[0] & 0b0000_0001;
        let ra =     self.flags[1] & 0b1000_0000;
        let z =      0b0000_0000;
        let rcode =  if opcode == 0u8 { 0u8 } else { 4u8 };
        self.flags = [qr+opcode+aa+tc+rd, ra+z+rcode]
    }
    fn set_response_indicator(&mut self) {
        self.flags[0] |= 0b1000_0000;
    }

    fn as_vec(&self) -> Vec<u8> {
        [self.id, self.flags, self.qdcount, self.ancount, self.nscount, self.arcount].concat()
    }
}

impl DnsQuestion {
    fn from(buf: &[u8]) -> Self {
        let mut domain_name: Vec<u8> = buf.iter().take_while(|x| **x != 0u8).cloned().collect();
        domain_name.push(0u8);
        DnsQuestion {
            qname: domain_name,
            qtype: 1u16.to_be_bytes(),
            qclass: 1u16.to_be_bytes(),
        }
    }
    fn as_vec(&self) -> Vec<u8> {
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
            header: DnsHeader::from(&buf[..12]),
            question: DnsQuestion::from(&buf[12..]),
        }
    }

    fn response(&mut self) -> Vec<u8> {
        self.header.parse_flags();
        self.header.set_response_indicator();
        self.header.qdcount = 1u16.to_be_bytes();
        self.header.ancount = 1u16.to_be_bytes();
        [
            self.header.as_vec(),
            self.question.as_vec(),
            DnsAnswer::new().as_vec(),
        ].concat()
    }
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
                println!("{:?}", message.header.parse_flags());
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
