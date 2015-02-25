/*
Gordon Adam
1107425

Struct that represents a message made up of a header, and possible questions and resource records
*/


use std::default;
use std::io::BufReader;
use std::io::net::ip::{SocketAddr};
use question::Question;
use resource::Resource;
use header::Header;

#[deriving(Default,Clone)]
pub struct Message {
    pub header: Header, // The header of the message
    pub questions: Vec<Question>, // The vector of questions
    pub answers: Vec<Resource>, // The vector of answer resource records
    pub authority: Vec<Resource>, // The vector of authority resource records
    pub additional: Vec<Resource>, // The vector of additional resource records
    pub msg_copy: Vec<u8>, // A copy of the message stored as a vector of u8 characters
    pub timestamp: i64,
    pub server: uint,
}

impl Message {

	// Creates a new message with default values
	pub fn new() -> Message {
		return Message {..default::Default::default()};
	}

	// reads into the struct from a vector of u8 characters provided
	pub fn read_in(&mut self, data: &mut [u8], length: uint) -> Result<(), String>{

		self.make_copy(data, length);

		let mut reader = BufReader::new(data.slice_to(length));

		self.header = Header::new();

		self.header.read_in(&mut reader);

		for i in range(0u, self.header.qdcount.to_uint().unwrap()) {
			self.questions.push(Question::new());
			match self.questions[i].read_in(&mut reader, &mut self.msg_copy) {
				Ok(()) => {},
				Err(a) => {return Err(a)}
			}
		}

		for i in range(0u, self.header.ancount.to_uint().unwrap()) {
			self.answers.push(Resource::new());
			match self.answers[i].read_in(&mut reader, &mut self.msg_copy) {
				Ok(()) => {},
				Err(a) => {return Err(a)}
			}
		}

		for i in range(0u, self.header.nscount.to_uint().unwrap()) {
			self.authority.push(Resource::new());
			match self.authority[i].read_in(&mut reader, &mut self.msg_copy) {
				Ok(()) => {},
				Err(a) => {return Err(a)}
			}
		}

		for i in range(0u, self.header.arcount.to_uint().unwrap()) {
			self.additional.push(Resource::new());
			match self.additional[i].read_in(&mut reader, &mut self.msg_copy) {
				Ok(()) => {},
				Err(a) => {return Err(a)}
			}
		}

		return Ok(());
	}

	// generates a query message for the hostname provided
	pub fn generate_query(&mut self, name: Vec<u8>) {
		self.header.generate_query_header();
		self.questions.push(Question::new());
		self.questions[0].generate(name, &mut self.msg_copy);
	}

	// creates and returns a vector of u8 characters made up from the message
	pub fn write(&mut self) -> Vec<u8> {

		let mut message_buffer: Vec<u8> = vec![];

    	message_buffer.push_all(self.header.write().as_slice());

    	for i in range(0u, self.header.qdcount.to_uint().unwrap()) {
    		message_buffer.push_all(self.questions[i].write().as_slice());
    	}

    	for i in range(0u, self.header.ancount.to_uint().unwrap()) {
			message_buffer.push_all(self.answers[i].write().as_slice());
		}

    	return message_buffer;
	}

	// Returns the next ipv4 address in the additional records, if no more exist return none
	pub fn next_server(&mut self) -> Option<SocketAddr> {
		while self.server < self.additional.len() {

			match self.additional[self.server].ip_addr() {
				Some(ip) => {
					self.server = self.server + 1;
					return Some(ip)
				},
				None => {
					self.server = self.server + 1;
				}
			}
		}
		return None;
	}

	// Prints out the entire message; including the header and each answer, name server and additional record attached to that particular message
	pub fn print(&mut self) {
		self.header.print();
		for i in range(0u, self.header.qdcount.to_uint().unwrap()) {
			self.questions[i].print();
		}
		
		println!("Answers");
		for i in range(0u, self.header.ancount.to_uint().unwrap()) {
			self.answers[i].print();
		}
		println!("Authoritative Name Servers");
		for i in range(0u, self.header.nscount.to_uint().unwrap()) {
			self.authority[i].print();
		}
		
		println!("Additional Records");
		for i in range(0u, self.header.arcount.to_uint().unwrap()) {
			self.additional[i].print();
		}
		
	}

	pub fn drop_records(&mut self) {
		if self.header.ancount > 0 {
			self.answers.clear();
		}
		/*
		if(self.header.nscount > 0) {
			self.authority.clear();
		}
		if(self.header.arcount > 0) {
			self.additional.clear();
		}
		*/
		self.header.ancount = 0x0000;
		/*
		self.header.nscount = 0x0000;
		self.header.arcount = 0x0000;
		self.server = 0;
		*/
	}

	// makes a copy of the message and stores it in the struct
	pub fn make_copy(&mut self, data: &mut [u8], length: uint) {
		for i in range(0, length) {
			self.msg_copy.push(data[i]);
		}
	}

	pub fn contains_type(&mut self, t: u16) -> bool {
		for i in range(0u16, self.header.ancount) {
			if self.answers[i as uint].rtype == t {
				return true;
			}
		}
		return false;
	}
}

