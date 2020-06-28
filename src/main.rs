use std::net::UdpSocket;
use ux;
use byteorder::{WriteBytesExt, ByteOrder, BigEndian}; // 1.3.4
use std::convert::TryInto;

#[derive(Debug)]
struct Message {
    header: MessageHeader,
    questions: Vec<Question>,
    responses: Vec<Response>,
    authority_records: Vec<Response>,
    additional_records: Vec<Response>
}

#[derive(Debug)]
struct MessageHeader {
    Id: u16,
    QR: bool,
    OpCode: ux::u4,
    AA: bool,
    TC: bool,
    RD: bool,
    RA: bool,
    Z: ux::u3,
    RCode: ux::u4,
    QDCount: u16,
    ANCount: u16,
    NSCount: u16,
    ARCount: u16
}

#[derive(Debug)]
struct Question {
    QName: Vec<Vec<u8>>,
    QType: u16,
    QClass: u16
}

#[derive(Debug)]
struct Response {
    Name: Vec<String>,
    Type: u16,
    Class: u16,
    TTL: u32,
    RDLength: u16,
    RData: Vec<u8>
}
/* Util Functions */
fn get_bit_at(input: u8, n: u8) -> bool {
    if n < 8 {
        input & (1 << n) != 0
    } else {
        false
    }
}

fn set_bit_at(input: u8, n: u8, value: bool) -> u8 {
    if get_bit_at(input, n) != value {
        return input ^ (1 << n)
    } else {
        return input
    }
}

fn bufToU16(bytes: [u8;2]) -> u16 {
    return BigEndian::read_u16(&bytes);
}

fn extractOpCode(byte: u8) -> ux::u4 {
    return ux::u4::new((byte << 1) >> 5)
}

fn extractZ(byte: u8) -> ux::u3 {
   return ux::u3::new((byte << 1) >> 5)
}

fn convert_u16_to_two_u8s_be(integer: u16) -> Vec<u8> {
    let mut res = vec![];
    res.write_u16::<BigEndian>(integer).unwrap();
    res
}

fn parseQuestion(starting_byte_position: usize, message_bytes: &mut Vec<u8>) -> (Question, usize) {
    let mut labels = vec![];

    let mut byte_position = starting_byte_position;
    loop {
        println!("{}", byte_position);
        let length = message_bytes[byte_position];
        println!("length: {}", length);
        if length == 0 {
            break
        }
        
        labels.push(message_bytes[byte_position+1..byte_position+1+usize::from(length)].to_vec());

        byte_position += 1 + usize::from(length);
    }

    return (Question {
        QName: labels.to_vec(),
        QType: bufToU16(message_bytes[byte_position..byte_position+2].try_into().expect("size 2 buffer")),
        QClass: bufToU16(message_bytes[byte_position+2..byte_position+4].try_into().expect("size 2 buffer"))
    }, byte_position + 4)
}


fn serializeHeader(message_header: MessageHeader) -> Vec<u8> {
    let mut serializedHeader = vec![];
    serializedHeader.append(&mut convert_u16_to_two_u8s_be(message_header.Id));

    let qr_byte = 0;
    let u8_opcode = u8::from(message_header.OpCode);
    u8_opcode << 3;


    return serializedHeader;
}

fn serializeMessage(message: Message) -> Vec<u8> {
    let mut serializedMessage = vec![];
    serializedMessage.append(&mut serializeHeader(message.header));


    return serializedMessage
}


fn parseMessage(bytes: &mut Vec<u8>) -> Message {
    let header_bytes = &mut bytes[..12];

    //println!("{}", header_bytes[3]);
    println!("{}", extractZ(header_bytes[3]));

    let header = MessageHeader {
        Id: bufToU16(header_bytes[..2].try_into().expect("size 2 buffer")),
        QR: get_bit_at(header_bytes[2], 7),
        OpCode: extractOpCode(header_bytes[2]),
        AA: get_bit_at(header_bytes[2], 2),
        TC: get_bit_at(header_bytes[2], 1),
        RD: get_bit_at(header_bytes[2], 0),
        RA: get_bit_at(header_bytes[3], 7),
        Z: extractZ(header_bytes[3]),
        RCode: ux::u4::new((header_bytes[3] << 4) >> 4),
        QDCount: bufToU16(header_bytes[4..6].try_into().expect("size 2 buffer")),
        ANCount: bufToU16(header_bytes[6..8].try_into().expect("size 2 buffer")),
        NSCount: bufToU16(header_bytes[8..10].try_into().expect("size 2 buffer")),
        ARCount: bufToU16(header_bytes[10..12].try_into().expect("size 2 buffer")),
    };

    let mut byte_position : usize = 12;
    let mut questions = vec![];
    for _ in 1..=header.QDCount {
        let (question, new_byte_position) = parseQuestion(byte_position, bytes);
        byte_position = new_byte_position;
        questions.push(question)
    }

    return Message {
        header: header,
        questions: questions,
        responses: vec![],
        authority_records: vec![],
        additional_records: vec![]
    }
}

fn main() {
  let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
  // Not sure how big to make this buffer, is there a max
  // size on domain names?  
  let mut buf = [0; 100];
  loop {
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
                                        .expect("Didn't receive data");
    let filled_buf = &mut buf[..number_of_bytes];

    let message = parseMessage(&mut filled_buf.to_vec());

    println!("{:?}", message);
  }
}