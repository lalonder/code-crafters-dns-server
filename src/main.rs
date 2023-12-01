// Uncomment this block to pass the first stage
use std::net::UdpSocket;

struct DnsMessage {
    id: &[str] = b"1234",

}

fn main() -> std::io::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    let udp_socket = UdpSocket::bind("127.0.0.1:2053")?;
    let mut buf = [0; 12];
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let headers = String::from_utf8_lossy(&buf[0..size]);
                println!("Received {} bytes from {}: {}", size, source, &headers);
                let response = headers;
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
