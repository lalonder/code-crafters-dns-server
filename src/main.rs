// Uncomment this block to pass the first stage
use std::net::UdpSocket;

#[derive(Debug)]
struct DnsMessage {
    header: [u8; 12]
}

impl DnsMessage {
    fn respond(&mut self) {
        self.header[2] += 0b1000_0000u8;
    }
}


fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to socket.");
    let mut buf = [0; 512];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let mut message = DnsMessage { header: <[u8; 12]>::try_from(&buf[..12]).unwrap() };
                println!("Received {} bytes from {}: {:?}", size, source, message.header);
                message.respond();
                udp_socket
                    .send_to(&message.header, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
