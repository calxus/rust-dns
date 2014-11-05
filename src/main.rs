#![feature(slicing_syntax)]
#![feature(macro_rules)]

use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::BufReader;

macro_rules! unwrap(($e: expr) => (match $e { Ok(v) => v, Err(e) => {fail!("Error")} }))

struct Message {
    header: Header,
    questions: Vec<Question>,
    answers: Vec<Resource>,
    authority: Vec<Resource>,
    additional: Vec<Resource>,
}

struct Header {
    id: u16,
    qr: u16,
    opcode: u16,
    aa: u16,
    tc: u16,
    rd: u16,
    ra: u16,
    rcode: u16,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

struct Name {
    label: Vec<String>,
    length: Vec<u8>,
}

struct Question {
    qname: Name,
    qtype: u16,
    qclass: u16,
}

struct Resource {
    rname: Name,
    rtype: u16,
    rclass: u16,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

fn main() {
    let addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 53 };
    let mut socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => fail!("couldn't bind socket: {}", e),
    };
    let mut buffer = [0, ..512];
    loop {
        match socket.recv_from(buffer) {
            Ok((length, src)) => {
                let mut message = read_request(&mut buffer, length);
                generate_response(&mut message);
                write_response(&mut message, src, &mut socket);
            },
            Err(e) => println!("couldn't receive a datagram: {}", e)
        }
    }
}

fn read_request(buffer: &mut [u8, ..512], length: uint) -> Message {

    let mut reader = BufReader::new(buffer.slice_to(length));

    let mut message = Message {

        header: Header{

            id: 0, 
            qr: 0, 
            opcode: 0, 
            aa: 0, 
            tc: 0, 
            rd: 0, 
            ra: 0, 
            rcode: 0, 
            qdcount: 0, 
            ancount: 0, 
            nscount: 0, 
            arcount: 0
        }, 

        questions: vec![], 
        answers: vec![], 
        authority: vec![], 
        additional: vec![]
    };

    let mut u16_buffer= unwrap!(reader.read_be_u16());
    message.header.id = u16_buffer.clone();

    u16_buffer = unwrap!(reader.read_be_u16());

    if (u16_buffer & 0x8000) == 0x8000 {
        message.header.qr = 0x8000;
    } else {
        message.header.qr = 0x0000;
    };

    match u16_buffer& 0x7800 {
        0x0000 => {message.header.opcode = 0x0000;},
        0x0800 => {message.header.opcode = 0x0800;},
        0x1000 => {message.header.opcode = 0x1000;},
        _ => fail!("Invalid OpCode"),
    };

    if ((u16_buffer & 0x0780) == 0x0400) { message.header.aa = 0x0400 };
    if ((u16_buffer & 0x0780) == 0x0200) { message.header.tc = 0x0200 };
    if ((u16_buffer & 0x0780) == 0x0100) { message.header.rd = 0x0100 };
    if ((u16_buffer & 0x0780) == 0x0080) { message.header.ra = 0x0080 };

    match u16_buffer & 0x000F {
        0x0000 => {message.header.rcode = 0x0000;},
        0x0001 => {message.header.rcode = 0x0001;},
        0x0002 => {message.header.rcode = 0x0002;},
        0x0003 => {message.header.rcode = 0x0003;},
        0x0004 => {message.header.rcode = 0x0004;},
        0x0005 => {message.header.rcode = 0x0005;},
        _ => fail!("Invalid RCode"),
    };

    u16_buffer = unwrap!(reader.read_be_u16());
    message.header.qdcount = u16_buffer;

    u16_buffer = unwrap!(reader.read_be_u16());
    message.header.ancount = u16_buffer;

    u16_buffer = unwrap!(reader.read_be_u16());
    message.header.nscount = u16_buffer;

    u16_buffer = unwrap!(reader.read_be_u16());
    message.header.arcount = u16_buffer;

    let mut question = Question {qname: Name {label: vec![], length: vec![]}, qtype: 0, qclass: 0};

    let mut byte_buffer = unwrap!(reader.read_u8());

    while (byte_buffer != 0) {

        let mut label: Vec<u8> = vec![];

        for i in range(0, byte_buffer)  {
            let mut temp_byte_buffer = unwrap!(reader.read_u8());
            label.push(temp_byte_buffer);
        }

        question.qname.label.push(unwrap!(String::from_utf8(label)));
        question.qname.length.push(byte_buffer);

        byte_buffer = unwrap!(reader.read_u8());
    }

    u16_buffer = unwrap!(reader.read_be_u16());
    question.qtype = u16_buffer;

    u16_buffer = unwrap!(reader.read_be_u16());
    question.qclass = u16_buffer;

    message.questions.push(question);

    return message;
}

fn generate_response(message: &mut Message) {

    let mut answer = Resource {rname: Name {label: vec![], length: vec![]}, rtype: 0, rclass: 0, ttl: 0, rdlength: 0, rdata: vec![]};

    answer.rname.label = message.questions[0].qname.label.clone();
    answer.rname.length = message.questions[0].qname.length.clone();
    answer.rtype = message.questions[0].qtype;
    answer.rclass = message.questions[0].qclass;
    answer.ttl = 0;
    answer.rdlength = 4;
    answer.rdata.push(74);
    answer.rdata.push(125);
    answer.rdata.push(230);
    answer.rdata.push(144);

    message.answers.push(answer);
    message.header.qr = 0x8000;
    message.header.ra = 0x0080;
    message.header.ancount = 0x0001;
}

fn write_response(message: &mut Message, src: SocketAddr, socket: &mut UdpSocket) {

    let mut message_buffer: Vec<u8> = vec![];

    split_u16(message.header.id, &mut message_buffer);

    let header_options: u16 = (
        message.header.qr | 
        message.header.opcode | 
        message.header.aa | 
        message.header.tc | 
        message.header.rd | 
        message.header.ra | 
        message.header.rcode
    );

    split_u16(header_options, &mut message_buffer);
    split_u16(message.header.qdcount, &mut message_buffer);
    split_u16(message.header.ancount, &mut message_buffer);
    split_u16(message.header.nscount, &mut message_buffer);
    split_u16(message.header.arcount, &mut message_buffer);

    for i in range(0, message.questions[0].qname.label.len()) {
        let mut question_buffer = message.questions[0].qname.label[i].clone().into_bytes();
        message_buffer.push(message.questions[0].qname.length[i]);
        for j in range(0, question_buffer.len()) {
            message_buffer.push(question_buffer[j]);
        }
    }
    message_buffer.push(0u8);

    split_u16(message.questions[0].qtype, &mut message_buffer);
    split_u16(message.questions[0].qclass, &mut message_buffer);

    for i in range(0, message.answers[0].rname.label.len()) {
        let mut answer_buffer = message.answers[0].rname.label[i].clone().into_bytes();
        message_buffer.push(message.answers[0].rname.length[i]);
        for j in range(0, answer_buffer.len()) {
            message_buffer.push(answer_buffer[j]);
        }
    }
    message_buffer.push(0u8);

    split_u16(message.answers[0].rtype, &mut message_buffer);
    split_u16(message.answers[0].rclass, &mut message_buffer);

    let ttl: u8 = 0;
    message_buffer.push(ttl);
    message_buffer.push(ttl);
    message_buffer.push(ttl);
    message_buffer.push(ttl);

    split_u16(message.answers[0].rdlength, &mut message_buffer);

    message_buffer.push(message.answers[0].rdata[0]);
    message_buffer.push(message.answers[0].rdata[1]);
    message_buffer.push(message.answers[0].rdata[2]);
    message_buffer.push(message.answers[0].rdata[3]);

    match socket.send_to(message_buffer.as_slice(), src) {
        Ok(r) => {},
        Err(e) => {},
    };   
           
}

fn split_u16(u: u16, message_buffer: &mut Vec<u8>) {
    message_buffer.push(((u & 0xFF00) >> 8) as u8);
    message_buffer.push(((u & 0x00FF) >> 0) as u8);
}














