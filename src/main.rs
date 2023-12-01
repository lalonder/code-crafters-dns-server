// Uncomment this block to pass the first stage
use std::net::UdpSocket;

#[derive(Debug)]
struct DnsMessage {
    header: [u8; 12]
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
                let headers = <[u8; 12]>::try_from(&buf[..12]).unwrap();
                let body = std::str::from_utf8(&buf[12..size]).expect("Failed to parse body");
                println!("Received {} bytes from {}: {:?} {}", size, source, headers, body);
                let response = DnsMessage { header: headers };
                udp_socket
                    .send_to(&response.header, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
