#![feature(slicing_syntax)]

use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};

struct message {
    ID: u16,
    QR: u16,
    OPCODE: u16,
    AA: u16,
    TC: u16,
    RD: u16,
    RA: u16,
    RCODE: u16,
    QDCOUNT: u16,
    ANCOUNT: u16,
    NSCOUNT: u16,
    ARCOUNT: u16,
}

struct question {
    label: Vec<String>,
    length: Vec<u8>,
    qtype: u16,
    qclass: u16,
}

struct answer {
    label: Vec<String>,
    length: Vec<u8>,
    TYPE: u16,
    CLASS: u16,
    TTL: u32,
    RDLENGTH: u16,
    RDATA: Vec<u8>,
}

fn main() {
    let mut msg = message {ID: 0, QR: 0, OPCODE: 0, AA: 0, TC: 0, RD: 0, RA: 0, RCODE: 0, QDCOUNT: 0, ANCOUNT: 0, NSCOUNT: 0, ARCOUNT: 0};
    
    let addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 53 };
    let mut socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => fail!("couldn't bind socket: {}", e),
    };
    let mut buf = [0, ..1000];
    loop {
        buf = [0, ..1000];
        match socket.recv_from(buf) {
            Ok((length, src)) => {
                let mut reader = std::io::BufReader::new(buf.slice_to(length));
                println!("\n========================\n---Header---");
                let mut x = reader.read_be_u16().unwrap();
                msg.ID = x;
                println!("ID: {}", x);
                x = reader.read_be_u16().unwrap();
                if (x & 0x8000) == 0x8000 {println!("QR: 1 (response)"); msg.QR = 0x8000;} else {println!("QR: 0 (query)"); msg.QR = 0x0000;};
                match x & 0x7800 {
                    0x0000 => {println!("OPCODE: 0000 (standard query)"); msg.OPCODE = 0x0000;},
                    0x0800 => {println!("OPCODE: 0001 (inverse query)"); msg.OPCODE = 0x0800;},
                    0x1000 => {println!("OPCODE: 0010 (server status request)"); msg.OPCODE = 0x1000;},
                    _ => println!("OPCODE: Reserved Operator")
                };
                match x & 0x0780 {
                    0x0400 => {println!("AA: 1 (Authoritative Answer)\nTC: 0\nRD: 0\nRA: 0"); msg.AA = 0x0400;},
                    0x0200 => {println!("AA: 0\nTC: 1 (TrunCation)\nRD: 0\nRA: 0"); msg.TC = 0x0200;},
                    0x0100 => {println!("AA: 0\nTC: 0\nRD: 1 (Recursion Desired)\nRA: 0"); msg.RD = 0x0100;},
                    0x0080 => {println!("AA: 0\nTC: 0\nRD: 0\nRA: 1 (Recursion Available)"); msg.RA = 0x0080;},
                    _ => println!("AA: 0\nTC: 0\nRD: 0\nRA: 0\n"),
                };
                match x & 0x000F {
                    0x0000 => {println!("RCODE: 0000 (No Error Condition)"); msg.RCODE = 0x0000;},
                    0x0001 => {println!("RCODE: 0001 (Format Error)"); msg.RCODE = 0x0001;},
                    0x0002 => {println!("RCODE: 0010 (Server Failure)"); msg.RCODE = 0x0002;},
                    0x0003 => {println!("RCODE: 0011 (Name Error)"); msg.RCODE = 0x0003;},
                    0x0004 => {println!("RCODE: 0100 (Not Implemented)"); msg.RCODE = 0x0004;},
                    0x0005 => {println!("RCODE: 0101 (Refused)"); msg.RCODE = 0x0005;},
                    _ => println!("RCODE: (Reserved for Future Use)"),
                };
                x = reader.read_be_u16().unwrap();
                msg.QDCOUNT = x;
                match x.to_uint() {
                    Some(n) => println!("QDCOUNT: {}", n),
                    None => fail!("boo"),
                };
                x = reader.read_be_u16().unwrap();
                msg.ANCOUNT = x;
                match x.to_uint() {
                    Some(n) => println!("ANCOUNT: {}", n),
                    None => fail!("boo"),
                };
                x = reader.read_be_u16().unwrap();
                msg.NSCOUNT = x;
                match x.to_uint() {
                    Some(n) => println!("NSCOUNT: {}", n),
                    None => fail!("boo"),
                };
                x = reader.read_be_u16().unwrap();
                msg.ARCOUNT = x;
                match x.to_uint() {
                    Some(n) => println!("ARCOUNT: {}", n),
                    None => fail!("boo"),
                };

                println!("\n---Question----");

                let mut qsn = question {label: vec![], length: vec![], qtype: 0, qclass: 0};

                let mut x8 = reader.read_u8().unwrap();

                while (x8 != 0) {

                    let mut label: Vec<u8> = vec![];

                    for i in range(0, x8)  {
                        let mut y = reader.read_u8().unwrap();
                        label.insert(0, y);
                    }

                    label.reverse();

                    qsn.label.insert(0, String::from_utf8(label).unwrap());
                    qsn.length.insert(0, x8);

                    x8 = reader.read_u8().unwrap();
                }

                qsn.label.reverse();
                qsn.length.reverse();

                for i in range(0, qsn.label.len()) {
                    println!("Label: {} (Length Octet: {})", qsn.label[i], qsn.length[i]);
                }

                x = reader.read_be_u16().unwrap();

                qsn.qtype = x;

                match x & 0xFFFF {
                    0x0001 => println!("QTYPE: A (a host address)"),
                    0x0002 => println!("QTYPE: NS (an authoritative name server)"),
                    0x0003 => println!("QTYPE: MD (a mail destination (OBSOLETE))"),
                    0x0004 => println!("QTYPE: MF (a mail forwarder (OBSOLETE))"),
                    0x0005 => println!("QTYPE: CNAME (the canonical name for an alias)"),
                    0x0006 => println!("QTYPE: SOA (marks the start of a zone of authority)"),
                    0x0007 => println!("QTYPE: MB (a mailbox domain name (EXPERIMENTAL))"),
                    0x0008 => println!("QTYPE: MG (a mail group member (EXPERIMENTAL))"),
                    0x0009 => println!("QTYPE: MR (a mail rename domain name (EXPERIMENTAL))"),
                    0x000A => println!("QTYPE: NULL (a null RR (EXPERIMENTAL))"),
                    0x000B => println!("QTYPE: WKS (a well known service description)"),
                    0x000C => println!("QTYPE: PTR (a domain name pointer)"),
                    0x000D => println!("QTYPE: HINFO (host information)"),
                    0x000E => println!("QTYPE: MINFO (mailbox or mail list information)"),
                    0x000F => println!("QTYPE: MX (mail exchange)"),
                    0x0010 => println!("QTYPE: TXT (text strings)"),
                    0x00FC => println!("QTYPE: AXFR (a request for a transfer of an entire zone)"),
                    0x00FD => println!("QTYPE: MAILB (a request for mailbox-related records)"),
                    0x00FE => println!("QTYPE: MAILA (a request for mail agent RRs)"),
                    0x00FF => println!("QTYPE: * (a request for all records)"),
                    _ => println!("QTYPE: INVALID ({})", x),
                };

                x = reader.read_be_u16().unwrap();

                qsn.qclass = x;

                match x & 0xFFFF {
                    0x0001 => println!("QCLASS: IN (the Internet)"),
                    0x0002 => println!("QCLASS: CS (the CSNET class (OBSOLETE))"),
                    0x0003 => println!("QCLASS: CH (the CHAOS class)"),
                    0x0004 => println!("QCLASS: HS (Hesiod)"),
                    0x00FF => println!("QCLASS: * (any class)"),
                    _ => println!("QCLASS: INVALID"),
                };

                let mut ans = answer {label: vec![], length: vec![], TYPE: 0, CLASS: 0, TTL: 0, RDLENGTH: 0, RDATA: vec![]};

                ans.label = qsn.label.clone();
                ans.length = qsn.length.clone();
                ans.TYPE = qsn.qtype;
                ans.CLASS = qsn.qclass;
                ans.TTL = 0;
                ans.RDLENGTH = 4;
                ans.RDATA.push(74);
                ans.RDATA.push(125);
                ans.RDATA.push(230);
                ans.RDATA.push(144);
                msg.QR = 0x8000;
                msg.RA = 0x0080;
                msg.ANCOUNT = 0x0001;

                let mut msgbuf: Vec<u8> = vec![];

                msgbuf.push(((msg.ID & 0xFF00) >> 8) as u8);
                msgbuf.push(((msg.ID & 0x00FF) >> 0) as u8);

                let msg2u16: u16 = (msg.QR | msg.OPCODE | msg.AA | msg.TC | msg.RD | msg.RA | msg.RCODE);

                msgbuf.push(((msg2u16 & 0xFF00) >> 8) as u8);
                msgbuf.push(((msg2u16 & 0x00FF) >> 0) as u8);

                msgbuf.push(((msg.QDCOUNT & 0xFF00) >> 8) as u8);
                msgbuf.push(((msg.QDCOUNT & 0x00FF) >> 0) as u8);

                msgbuf.push(((msg.ANCOUNT & 0xFF00) >> 8) as u8);
                msgbuf.push(((msg.ANCOUNT & 0x00FF) >> 0) as u8);

                msgbuf.push(((msg.NSCOUNT & 0xFF00) >> 8) as u8);
                msgbuf.push(((msg.NSCOUNT & 0x00FF) >> 0) as u8);

                msgbuf.push(((msg.ARCOUNT & 0xFF00) >> 8) as u8);
                msgbuf.push(((msg.ARCOUNT & 0x00FF) >> 0) as u8);

                for i in range(0, qsn.label.len()) {
                    let mut qsnbuf = qsn.label[i].clone().into_bytes();
                    msgbuf.push(qsn.length[i]);
                    for j in range(0, qsnbuf.len()) {
                        msgbuf.push(qsnbuf[j]);
                    }
                }
                msgbuf.push(0u8);

                msgbuf.push(((qsn.qtype & 0xFF00) >> 8) as u8);
                msgbuf.push(((qsn.qtype & 0x00FF) >> 0) as u8);

                msgbuf.push(((qsn.qclass & 0xFF00) >> 8) as u8);
                msgbuf.push(((qsn.qclass & 0x00FF) >> 0) as u8);

                for i in range(0, ans.label.len()) {
                    let mut ansbuf = ans.label[i].clone().into_bytes();
                    msgbuf.push(ans.length[i]);
                    for j in range(0, ansbuf.len()) {
                        msgbuf.push(ansbuf[j]);
                    }
                }

                msgbuf.push(0u8);

                msgbuf.push(((ans.TYPE & 0xFF00) >> 8) as u8);
                msgbuf.push(((ans.TYPE & 0x00FF) >> 0) as u8);

                msgbuf.push(((ans.CLASS & 0xFF00) >> 8) as u8);
                msgbuf.push(((ans.CLASS & 0x00FF) >> 0) as u8);

                let ttl: u8 = 0;

                msgbuf.push(ttl);
                msgbuf.push(ttl);
                msgbuf.push(ttl);
                msgbuf.push(ttl);

                msgbuf.push(((ans.RDLENGTH & 0xFF00) >> 8) as u8);
                msgbuf.push(((ans.RDLENGTH & 0x00FF) >> 0) as u8);

                msgbuf.push(ans.RDATA[0]);
                msgbuf.push(ans.RDATA[1]);
                msgbuf.push(ans.RDATA[2]);
                msgbuf.push(ans.RDATA[3]);

                match socket.send_to(msgbuf.as_slice(), src) {
                    Ok(r) => {},
                    Err(e) => {},
                };

            },
            Err(e) => println!("couldn't receive a datagram: {}", e)
        }
    }
    drop(socket); // close the socket
}