use std::net::UdpSocket;
use bytes::BytesMut;
use nom::IResult;
use nom::bytes::complete::{take, take_until};
use nom::sequence::tuple;

#[derive(Debug, Copy, Clone)]
struct Header {
    id: [u8; 2],
    flags: [u8; 2],
    qdcount: [u8; 2],
    ancount: [u8; 2],
    nscount: [u8; 2],
    arcount: [u8; 2],
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

    fn set_response_indicator(&mut self) {
        self.flags[0] |= 0b1000_0000;
    }

    fn to_vec(self) -> Vec<u8> {
        [
            self.id, self.flags, self.qdcount, self.ancount, self.nscount, self.arcount
        ].concat()
    }
}

impl Question {
    fn from(buf: &[u8]) -> Vec<Question> {
        let mut output: Vec<Question> = Vec::new();
        {
            output.push(parse_question(buf).unwrap().1);
        }
        output
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
            questions: Question::from(&buf[12..]),
            answers: {
                vec![Answer::from(&buf[12..])]
            },
        }
    }

    fn response(&mut self) -> Vec<u8> {
        self.header.set_response_indicator();
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

fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (
        input,
        (id, flags, qdcount, ancount, nscount, arcount)
    ) = tuple((take_two_bytes, take_two_bytes, take_two_bytes, take_two_bytes, take_two_bytes, take_two_bytes))(input)?;
    Ok((
        input,
        Header { id, flags, qdcount, ancount, nscount, arcount, }
    ))
}

fn parse_question(input: &[u8]) -> IResult<&[u8], Question> {
    let (input, (qname, qtype, qclass)) = tuple((until_null, take_two_bytes, take_two_bytes))(input)?;
    Ok((
        input,
        Question { qname: Vec::from(qname), qtype, qclass, }
    ))
}

fn take_two_bytes(input: &[u8]) -> IResult<&[u8], [u8; 2]> {
    let (input, bytes) = take(2usize)(input)?;
    Ok((input, [bytes[0], bytes[1]]))
}

fn until_null(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_until("\x00")(input)
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
