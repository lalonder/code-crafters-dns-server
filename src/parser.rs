use nom::bytes::complete::{take, take_until};
use nom::IResult;
use nom::combinator::{map_parser};
use nom::multi::{length_data, many0};
use nom::sequence::tuple;
use nom::number::complete::{be_u8, be_u16};
use crate::{Header, Question};
use crate::parser::bits::parse_flags;

pub fn parse_message(input: &[u8]) -> IResult<&[u8], (Header, Question)> {
    tuple((parse_header, parse_question))(input)
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    match tuple((
        be_u16,
        parse_flags,
        be_u16,
        be_u16,
        be_u16,
        be_u16,
    ))(input) {
        Ok((input, (id, flags, qdcount, ancount, nscount, arcount))) =>
            Ok((
                input,
                Header { id, flags, qdcount, ancount, nscount, arcount, },
            )),
        _ => panic!(),
    }
}

pub fn parse_question(input: &[u8]) -> IResult<&[u8], Question> {
    if input[0] != 0x0 {
        match tuple((
            map_parser(until_null, take_label),
            take(1usize),
            be_u16,
            be_u16,
        ))(input) {
            Ok((input, (qname, _null, qtype, qclass))) => {
                let mut qname = Vec::from(qname);
                qname.push(0x0);
                Ok((input, Question { qname, qtype, qclass }))
            },
            _ => panic!(),
        }
    } else {
        Ok((input, Question::new()))
    }
}

pub fn parse_questions(input: &[u8]) -> IResult<&[u8], Vec<Question>> {
    many0(parse_question)(input)
}

fn until_null(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_until("\x00")(input)
}

fn take_label(input: &[u8]) -> IResult<&[u8], &[u8]> {
    length_data(be_u8)(input)
}

pub mod bits {
    use nom::IResult;
    use nom::error::Error;
    use nom::sequence::tuple;
    use nom::bits::complete::take;
    use nom::bits::bits;
    use crate::Flags;

    type IResultFlags<'a> = (&'a [u8], (u8, u8, u8, u8, u8, u8, u8, u8));

    pub fn parse_flags(input: &[u8]) -> IResult<&[u8], Flags> {
        let (input, (qr, opcode, aa, tc, rd, ra, z, _rcode)): IResultFlags =
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
                rcode: match opcode {
                    0x0 => 0x0,
                    _ => 0x4
                },
            }
        ))
    }
}