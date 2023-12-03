use std::net::UdpSocket;

#[derive(Debug)]
struct DnsHeader {
    id: u16,
    flags: [u8; 2],
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

#[derive(Debug)]
struct DnsMessage {
    header: DnsHeader,
}

impl From<&[u8]> for DnsHeader {
    fn from(buffer: &[u8]) -> Self {
        DnsHeader {
            id:         u16::from_be_bytes([buffer[0], buffer[1]]),
            flags:      [buffer[2], buffer[3]],
            qdcount:    u16::from_be_bytes([buffer[4], buffer[5]]),
            ancount:    u16::from_be_bytes([buffer[6], buffer[7]]),
            nscount:    u16::from_be_bytes([buffer[8], buffer[9]]),
            arcount:    u16::from_be_bytes([buffer[10], buffer[11]]),
        }
    }
}

fn get_header_array(header: &DnsHeader) -> Vec<u8> {
    [
        header.id.to_be_bytes(),
        header.flags,
        header.qdcount.to_be_bytes(),
        header.ancount.to_be_bytes(),
        header.nscount.to_be_bytes(),
        header.arcount.to_be_bytes(),
    ].concat()
}

impl From<&[u8]> for DnsMessage {
    fn from(buffer: &[u8]) -> Self {
        DnsMessage {
            header: DnsHeader::from(&buffer[..12]),
        }
    }
}

impl DnsHeader {
    fn set_response_flag(&mut self) {
        self.flags[0] += 0b1000_0000
    }
}

fn main() {
    println!("Logs from your program will appear here!");

    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to socket.");
    let mut buf = [0; 512];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let mut message = DnsMessage::from(&buf[..]);
                println!("Received {} bytes from {}: {:?}", size, source, get_header_array(&message.header));
                message.header.set_response_flag();
                println!("{:?}", message);
                udp_socket
                   .send_to(&get_header_array(&message.header)[..], source)
                   .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
