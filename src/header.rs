/*
Gordon Adam
1107425

Struct to represent the header of a message
*/

use std::default;
use std::io::BufReader;
use std::rand;


#[deriving(Default,Clone)]
pub struct Header {
    pub id: u16, // The ID number of the header eg. 0x997F
    // The following variables from qr -> rcode are combined when they are transmitted
    pub qr: u16, // Question or Response bit
    pub opcode: u16, // 4-bit opcode states the type of request
    pub aa: u16, // Authoritative bit, states whether the response is from an authority for that domain
    pub tc: u16, // Truncated bit states whether the message is truncated or not
    pub rd: u16, // Recursion desired bit states whether recursion is desired or not
    pub ra: u16, // Recursion available bit states whether recursion is made available by the server or not
    pub rcode: u16, // 4-bit reply code states whether there is some error with the request made
    pub qdcount: u16, // Question count 
    pub ancount: u16, // Answer count
    pub nscount: u16, // Authoritative Name Servers count
    pub arcount: u16, // Additional Records count 
}

impl Header {

    // Creates an instance of the header struct with default values
    pub fn new() -> Header {
        return Header {..default::Default::default()}
    }

    // Reads in the values to the struct from a buffered reader
    pub fn read_in(&mut self, reader: &mut BufReader) {

        self.id = reader.read_be_u16().unwrap();

        let u16_buffer = reader.read_be_u16().unwrap();

        if (u16_buffer & 0x8000) == 0x8000 {self.qr = 0x8000;} else {self.qr = 0x0000;};

        match u16_buffer & 0x7800 {
            0x0000 => {self.opcode = 0x0000;},
            0x0800 => {self.opcode = 0x0800;},
            0x1000 => {self.opcode = 0x1000;},
            _ => {},
        };

        if (u16_buffer & 0x0400) == 0x0400 { self.aa = 0x0400 };
        if (u16_buffer & 0x0200) == 0x0200 { self.tc = 0x0200 };
        if (u16_buffer & 0x0100) == 0x0100 { self.rd = 0x0000 };
        if (u16_buffer & 0x0080) == 0x0080 { self.ra = 0x0080 };

        match u16_buffer & 0x000F {
            0x0000 => {self.rcode = 0x0000;},
            0x0001 => {self.rcode = 0x0001;},
            0x0002 => {self.rcode = 0x0002;},
            0x0003 => {self.rcode = 0x0003;},
            0x0004 => {self.rcode = 0x0004;},
            0x0005 => {self.rcode = 0x0005;},
            _ => {},
        };

        self.qdcount = reader.read_be_u16().unwrap();
        self.ancount = reader.read_be_u16().unwrap();
        self.nscount = reader.read_be_u16().unwrap();
        self.arcount = reader.read_be_u16().unwrap();
    }

    // generates a generic header for a query message
    pub fn generate_query_header(&mut self) {
        self.id = rand::random::<u16>();
        self.qr = 0x0000;
        self.opcode = 0x0000;
        self.aa = 0x0000;
        self.tc = 0x0000;
        self.rd = 0x0000;
        self.ra = 0x0000;
        self.rcode = 0x0000;
        self.qdcount = 0x0001;
        self.ancount = 0x0000;
        self.nscount = 0x0000;
        self.arcount = 0x0000;
    }

    // creates and returns a vector of 8-bit characters made up from the values in the struct
    pub fn write(&mut self) -> Vec<u8> {

        self.tc = 0x0000;

        let mut header_buffer: Vec<u8> = vec![];

        split_u16(self.id, &mut header_buffer);

        let header_options: u16 = 
            self.qr | 
            self.opcode | 
            self.aa | 
            self.tc | 
            self.rd | 
            self.ra | 
            self.rcode;

        split_u16(header_options, &mut header_buffer);
        split_u16(self.qdcount, &mut header_buffer);
        split_u16(self.ancount, &mut header_buffer);
        split_u16(0x0000, &mut header_buffer);
        split_u16(0x0000, &mut header_buffer);

        return header_buffer;
    }
    // prints out the header in the form specified below as an example
    /*
    Transaction ID: 0x997F
    0... .... .... .... = Response: Message is a request
    .000 0... .... .... = Opcode: Standard query (0)
    .... .0.. .... .... = Authoritative: Server is not an authority for domain
    .... ..0. .... .... = Truncated: Message is not truncated
    .... ...1 .... .... = Recursion desired: Do query recursively
    .... .... 0... .... = Recursion available: Server can't do recursive queries
    .... .... .000 .... = Z: reserved (0)
    .... .... .... 0000 = Reply code: No error (0)
    Questions: 1
    Answer RRs: 0
    Authority RRs: 0
    Additional RRs: 0
    */
    pub fn print(&mut self) {
        println!("");
        println!("Transaction ID: 0x{:X}", self.id);

        if (self.qr & 0x8000) == 0x8000 {
            println!("1... .... .... .... = Response: Message is a response");
        } else {
            println!("0... .... .... .... = Response: Message is a request");
        };

        match self.opcode & 0x7800 {
            0x0000 => {println!(".000 0... .... .... = Opcode: Standard query (0)");},
            0x0800 => {println!(".000 1... .... .... = Opcode: Inverse query (1)");},
            0x1000 => {println!(".001 0... .... .... = Opcode: Server status request (2)");},
            _ => {},
        };

        if (self.aa & 0x0780) == 0x0400 {
            println!(".... .1.. .... .... = Authoritative: Server is an authority for domain");
        } else {
            println!(".... .0.. .... .... = Authoritative: Server is not an authority for domain");
        };

        if (self.tc & 0x0780) == 0x0200 {
            println!(".... ..1. .... .... = Truncated: Message is truncated");
        } else {
            println!(".... ..0. .... .... = Truncated: Message is not truncated");
        };

        if (self.rd & 0x0780) == 0x0100 {
            println!(".... ...1 .... .... = Recursion desired: Do query recursively");
        } else {
            println!(".... ...0 .... .... = Recursion desired: Do not query recursively");
        };

        if (self.ra & 0x0780) == 0x0080 { 
            println!(".... .... 1... .... = Recursion available: Server can do recursive queries");
        } else {
            println!(".... .... 0... .... = Recursion available: Server can't do recursive queries");
        };

        println!(".... .... .000 .... = Z: reserved (0)");

        match self.rcode & 0x000F {
            0x0000 => {println!(".... .... .... 0000 = Reply code: No error (0)");},
            0x0001 => {println!(".... .... .... 0001 = Reply code: Format Error (1)");},
            0x0002 => {println!(".... .... .... 0010 = Reply code: Server failure (2)");},
            0x0003 => {println!(".... .... .... 0011 = Reply code: Name error (3)");},
            0x0004 => {println!(".... .... .... 0100 = Reply code: Not implemented (4)");},
            0x0005 => {println!(".... .... .... 0101 = Reply code: Refused (5)");},
            _ => {},
        };

        println!("Questions: {}", self.qdcount);
        println!("Answer RRs: {}", self.ancount);
        println!("Authority RRs: {}", self.nscount);
        println!("Additional RRs: {}", self.arcount);
    }
}

// Splits a u16 character into two u8 characters and pushes them to a buffer provided
pub fn split_u16(u: u16, message_buffer: &mut Vec<u8>) {
    message_buffer.push(((u & 0xFF00) >> 8) as u8);
    message_buffer.push(((u & 0x00FF) >> 0) as u8);
}