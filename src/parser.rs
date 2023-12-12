use nom::bytes::complete::{take, take_until};
use nom::IResult;
use nom::sequence::tuple;
use nom::number::complete::be_u16;
use crate::{Header, Question};

// pub fn parse_flags(bytes: &[u8; 2]) -> [u8; 2] {
//     let qr =     bytes[0] & 0b1000_0000;
//     let opcode = bytes[0] & 0b0111_1000;
//     let aa =     0b0000_0000u8;
//     let tc =     0b0000_0000u8;
//     let rd =     bytes[0] & 0b0000_0001;
//
//     let ra =     bytes[1] & 0b1000_0000;
//     let z =      0b0000_0000;
//     let rcode =  if opcode == 0u8 { 0u8 } else { 4u8 };
//     [qr+opcode+aa+tc+rd, ra+z+rcode]
// }

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (
        input,
        (id, flags, qdcount, ancount, nscount, arcount,),
    ) = tuple((
        be_u16, take(2usize), be_u16, be_u16, be_u16, be_u16,
    ))(input)?;
    Ok((
        input,
        Header {
            id,
            flags: match bits::parse_flags(&flags) {
                Ok((_, flags)) => flags,
                _ => panic!(),
            },
            qdcount,
            ancount,
            nscount,
            arcount,
        }
    ))
}

pub fn parse_question(input: &[u8]) -> IResult<&[u8], Question> {
    let (
        input,
        (mut qname, qtype, qclass)
    ) = tuple((until_null, be_u16, be_u16))(input)?;
    qname.push(0x0);
    Ok((
        input,
        Question { qname, qtype, qclass }
    ))
}

fn until_null(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, output) = take_until("\x00")(input)?;
    Ok((input, Vec::from(output)))

}


pub mod bits {
    use nom::IResult;
    use nom::error::Error;
    use nom::sequence::tuple;
    use nom::bits::complete::take;
    use nom::bits::bits;
    use crate::Flags;

    pub fn parse_flags(input: &[u8]) -> IResult<&[u8], Flags> {
        type IResultFlags<'a> = (&'a [u8], (u8, u8, u8, u8, u8, u8, u8, u8));
        let (input, (qr, opcode, aa, tc, rd, ra, z, rcode)): IResultFlags =
            bits::<_, _, Error<(&[u8], usize)>, _, _,>(
            tuple((
                take(1usize),
                take(4usize),
                take(1usize),
                take(1usize),
                take(1usize),
                take(1usize),
                take(3usize),
                take(4usize),
            )))(input)?;
        Ok((
            input,
            Flags {
                qr: qr << 7,
                opcode: opcode << 3,
                aa: aa << 2,
                tc: tc << 1,
                rd,
                ra: ra << 7,
                z: z << 4,
                rcode,
            }
        ))
    }
}