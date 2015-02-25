/*
Gordon Adam
1107425

Struct to represent a question
*/

use std::default;
use std::io::BufReader;
use data;


#[deriving(Default,Clone)]
pub struct Question {
    pub qname: data::Data, // represents the hostname of the question
    pub qtype: u16, // the type of the question
    pub qclass: u16, // the class of the question
}

impl Question {

    // Creates an instance of the header struct with default values
	pub fn new() -> Question {
		return Question {..default::Default::default()};
	}

	// reads into the struct from a buffered reader 
	pub fn read_in(&mut self, reader: &mut BufReader, msg_copy: &mut Vec<u8>) -> Result<(), String> {

		self.qname = data::Data::new();
    	self.qname.read_hostname(reader, msg_copy);

    	self.qtype = reader.read_be_u16().unwrap();

    	if !((self.qtype != 0x0001) || (self.qtype != 0x0002)) {
    		return Err("Error: Question Type not supported".to_string());
    	}

    	self.qclass = reader.read_be_u16().unwrap();

    	return Ok(());
	}

	// generates a question for the name provided
	pub fn generate(&mut self, name: Vec<u8>, msg_copy: &mut Vec<u8>) {
		self.qname = data::Data::new();

		let mut reader = BufReader::new(name.as_slice());

		self.qname.read_hostname(&mut reader, msg_copy);

		self.qtype = 0x0001;
		self.qclass = 0x0001;
	}

	// converts the struct into a vector of u8 characters and returns it
	pub fn write(&mut self) -> Vec<u8> {
		let mut question_buffer: Vec<u8> = vec![];
		question_buffer.push_all(self.qname.write().as_slice());

    	split_u16(self.qtype, &mut question_buffer);
    	split_u16(self.qclass, &mut question_buffer);

    	return question_buffer;
	}

	// prints out the question in the specified format
	/*
	Queries
		Name: www.amazon.co.uk
		[Name Length: 13]
		[Label Count: 4]
		Type: A (Address record) (1)
		Class: IN (0x0001)
	*/
	pub fn print(&mut self) {
		println!("Queries");
		self.qname.print_as_query();
		print!("\t");
		match self.qtype & 0xFFFF {
			0x0001 => {println!("Type: A (Address record) (1)")},
			0x0002 => {println!("Type: NS (Name server record) (2)")},
			0x0005 => {println!("Type: CNAME (Canonical name record) (5)")},
			0x0006 => {println!("Type: SOA (Start of [a zone of] authority record) (6)")},
			0x000c => {println!("Type: PTR (Pointer record) (12)")},
			0x000f => {println!("Type: MX (Mail exchange record) (15)")},
			0x0010 => {println!("Type: TXT (Text record) (16)")},
			0x0011 => {println!("Type: RP (Responsible person) (17)")},
			0x0012 => {println!("Type: AFSDB (AFS database record) (18)")},
			0x0018 => {println!("Type: SIG (Signature) (24)")},
			0x0019 => {println!("Type: KEY (Key record) (25)")},
			0x001c => {println!("Type: AAAA (IPv6 address record) (28)")},
			_ => {println!("somethings wrong")}
		}
		print!("\t");
		match self.qclass & 0xFFFF {
			0x0001 => {println!("Class: IN (0x0001)")},
			0x0002 => {println!("Class: CS (0x0002)")},
			0x0003 => {println!("Class: CH (0x0003)")},
			0x0004 => {println!("Class: HS (0x0004)")},
			_ => {println!("somethings wrong")}
		}
	}
}

// Splits a u16 character into two u8 characters and pushes them to a buffer provided
pub fn split_u16(u: u16, message_buffer: &mut Vec<u8>) {
   	message_buffer.push(((u & 0xFF00) >> 8) as u8);
    message_buffer.push(((u & 0x00FF) >> 0) as u8);
}