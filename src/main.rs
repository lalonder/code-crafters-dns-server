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
struct DnsMessage {
    header: DnsHeader,
    question: DnsQuestion,
    // answer: Vec<u8>,
}

impl From<&[u8]> for DnsHeader {
    fn from(buffer: &[u8]) -> Self {
        DnsHeader {
            id:         [buffer[0], buffer[1]],
            flags:      [buffer[2], buffer[3]],
            qdcount:    [buffer[4], buffer[5]],
            ancount:    [buffer[6], buffer[7]],
            nscount:    [buffer[8], buffer[9]],
            arcount:    [buffer[10], buffer[11]],
        }
    }
}

impl From<&[u8]> for DnsQuestion {
    fn from(buf: &[u8]) -> Self {
        let mut domain_name: Vec<u8> = buf.iter().take_while(|x| **x != 0u8).cloned().collect();
        domain_name.push(0u8);
        DnsQuestion {
            qname: domain_name,
            qtype: 1u16.to_be_bytes(),
            qclass: 1u16.to_be_bytes(),
        }
    }
}

impl DnsHeader {
    fn set_response_flag(&mut self) {
        self.flags[0] ^= 0b1000_0000;
    }

    fn as_vec(&self) -> Vec<u8> {
        [self.id, self.flags, self.qdcount, self.ancount, self.nscount, self.arcount].concat()
    }
}

impl DnsQuestion {
    fn as_vec(&self) -> Vec<u8> {
        [&self.qname[..], &self.qtype[..], &self.qclass[..]].concat()
    }
}

impl DnsMessage {
    fn response(&mut self) -> Vec<u8> {
        self.header.id = [1234u16.to_be_bytes()[0], 1234u16.to_be_bytes()[1]];
        self.header.set_response_flag();
        self.header.qdcount = 1u16.to_be_bytes();
        [self.header.as_vec(), self.question.as_vec()].concat()
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to socket.");
    let mut buf = [0; 512];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let header = DnsHeader::from(&buf[..]);
                let question = DnsQuestion::from(&buf[12..]);
                let mut message = DnsMessage { header, question };
                let response = message.response();
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
