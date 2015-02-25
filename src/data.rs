/*
Gordon Adam
1107425

Struct that represents the data field of a resource.

Depending on the resource type, different types of data are stored within this struct
*/	


use std::default;
use std::io::BufReader;


#[deriving(Default,Clone,Show)]
pub struct Data {
    pub label: Vec<Vec<u8>>, // This is hold seperate strings of data ["www", "google", "com"]
    pub length: Vec<u8>, // This is the vector of lengths of the labels eg. ["3", "6", "3"]
}

impl Data {

	// Creates an instance of the struct with default values
	pub fn new() -> Data {
		return Data {..default::Default::default()};
	}

	// This function reads a name from the reader that may be compressed
	pub fn read_hostname(&mut self, reader: &mut BufReader, msg_copy: &mut Vec<u8>) {

		let mut byte_buffer = reader.read_u8().unwrap();
		let mut length: uint;
    	
    	while byte_buffer != 0 {
    		
    		if (byte_buffer & 0xc0) == 0xc0 {
    			length = ((byte_buffer - 0xc0) as uint) << 8;
    			byte_buffer = reader.read_u8().unwrap();
    			length = length + byte_buffer.to_uint().unwrap();
    			self.dereference_pointer(msg_copy, length);
    			break;
    		} else {
    			let mut label: Vec<u8> = vec![];
    			length = byte_buffer.to_uint().unwrap();
    			let mut i: uint = 0;
    			while i < length {
    				let temp_byte_buffer = reader.read_u8().unwrap();
    				label.push(temp_byte_buffer);
    				i = i + 1;
    			}
    			self.label.push(label.clone());
    			self.length.push(label.len() as u8);    			
    		}
    		byte_buffer = reader.read_u8().unwrap();
    	}
	}

	// Reads in the data section of an SOA record, this consists of a hostname, the email address of the responsible person
	// The serial number of the record, the refresh, retry, expire and minimum times of the record.
	pub fn read_soa(&mut self, reader: &mut BufReader, msg_copy: &mut Vec<u8>) {
		self.read_hostname(reader, msg_copy);
		self.length.push(0u8);
		self.label.push(vec![]);
		self.read_hostname(reader, msg_copy);
		self.length.push(0u8);
		self.label.push(vec![]);

		let mut byte_buffer: u8;

		for i in range(0u, 5) {
			let mut temp_label: Vec<u8> = vec![];
			for j in range(0u,4) {
				byte_buffer = reader.read_u8().unwrap();
				temp_label.push(byte_buffer);
			}
			self.label.push(temp_label);
		}
	}

	// This takes a copy of the message recieved and an index of that message and adds the name contained at that index to the data
	fn dereference_pointer(&mut self, msg_copy: &mut Vec<u8>, index: uint) {
		let mut idx = index;
		let mut length: uint;
		while msg_copy[idx] != 0 {
			if(msg_copy[idx] & 0xc0) == 0xc0 {
				idx = msg_copy[idx+1].to_uint().unwrap();
				self.dereference_pointer(msg_copy, idx);
				return;
			}
			self.length.push(msg_copy[idx]);
			length = msg_copy[idx].to_uint().unwrap();
			idx = idx + 1;
			let mut label: Vec<u8> = vec![];
			for i in range(0, length) {
				label.push(msg_copy[idx + i]);
			}
			self.label.push(label.clone());
			idx = idx + length;
		}
	}

	// Reads in an ipv4 address
	pub fn read_ipv4_addr(&mut self, reader: &mut BufReader) {
		let mut i: uint = 0;
		while i < 4u {
			let mut label: Vec<u8> = vec![];
			label.push(reader.read_u8().unwrap());
			self.label.push(label);
			self.length.push(1);
			i = i + 1;
		}
	}

	// Reads in an ipv6 address
	pub fn read_ipv6_addr(&mut self, reader: &mut BufReader) {
		let mut i: uint = 0;
		while i < 8u {
			let mut label: Vec<u8> = vec![];
			let mut j: uint = 0;
			while j < 2u {
				label.push(reader.read_u8().unwrap());
				j = j + 1;
			}
			self.label.push(label);
			self.length.push(2);
			i = i + 1;
		}
	}

	// Returns an ipv4 address
	pub fn get_ipv4_addr(&mut self) -> Vec<u8> {
		let mut addr: Vec<u8> = vec![];
		for i in range(0u, 4) {
			addr.push(self.label[i][0]);
		}
		return addr;
	}

	#[allow(dead_code)]
	pub fn get_ipv6_addr(&mut self) -> Vec<u8> {
		let mut addr: Vec<u8> = vec![];
		for i in range(0u, 8) {
			for j in range(0u, 2) {
				addr.push(self.label[i][j]);
			}
		}
		return addr;
	}

	// Returns the data as vector of u8 characters
	pub fn write(&mut self) -> Vec<u8> {
		let mut data_buffer: Vec<u8> = vec![];
		for i in range(0u, self.label.len()) {
			if i < self.length.len() {
				data_buffer.push(self.length[i]);
			}
			for j in range(0u, self.label[i].len()) {
				data_buffer.push(self.label[i][j]);
			}
		}
		data_buffer.push(0u8);
		return data_buffer;
	}

	// Returns cname data as a vector of u8 characters
	pub fn write_cname(&mut self) -> Vec<u8> {
		let mut data_buffer: Vec<u8> = vec![];
		for i in range(0u, self.label.len()) {
			data_buffer.push(self.length[i]);
			for j in range(0u, self.label[i].len()) {
				data_buffer.push(self.label[i][j]);
			}
		}
		data_buffer.push(0u8);
		return data_buffer;
	}


	pub fn write_ip_addr(&mut self) -> Vec<u8> {
		let mut data_buffer: Vec<u8> = vec![];
		for i in range(0u, self.label.len()) {
			for j in range(0u, self.label[i].len()) {
				data_buffer.push(self.label[i][j]);
			}
		}
		data_buffer.push(0u8);
		return data_buffer;
	}

	// function that will print a label out in the format "www.google.com"
	pub fn print_as_query(&mut self) {
		print!("\tName: ");
		for i in range(0, self.label.len()) {
			print!("{}", String::from_utf8(self.label[i].clone()).unwrap());
			if i<(self.label.len()-1) {
				print!(".");
			} else {
				println!("");
			}
		}
		let mut count = 0;
		for i in range(0, self.length.len()) {
			count = count + self.length[i];
		}
		println!("\t[Name Length: {}]", count);
		println!("\t[Label Count: {}]", self.label.len());
	}

	pub fn print_as_hostname(&mut self) {
		print!("Name: ");
		for i in range(0, self.label.len()) {
			print!("{}", String::from_utf8(self.label[i].clone()).unwrap());
			if i<(self.label.len()-1) {
				print!(".");
			} else {
				println!("");
			}
		}
	}

	pub fn print_as_soa(&mut self) {
		let mut idx = 0;
		print!("\n\tMName: ");
		while(self.length[idx] != 0u8) {
			print!("{}", String::from_utf8(self.label[idx].clone()).unwrap());
			idx = idx + 1;
			if self.length[idx] != 0u8 {
				print!(".")
			}
		}
		println!("");

		idx = idx + 1;
		print!("\tRName: ");
		while(self.length[idx] != 0u8) {
			print!("{}", String::from_utf8(self.label[idx].clone()).unwrap());
			idx = idx + 1;
			if self.length[idx] != 0u8 {
				print!(".")
			}
		}
		println!("");

		idx = idx + 1;
		let mut serial: u32 = self.u8_to_u32(idx);
		idx = idx + 1;
		let mut refresh: u32 = self.u8_to_u32(idx);
		idx = idx + 1;
		let mut retry: u32 = self.u8_to_u32(idx);
		idx = idx + 1;
		let mut expire: u32 = self.u8_to_u32(idx);
		idx = idx + 1;
		let mut minimum: u32 = self.u8_to_u32(idx);

		println!("\tSerial: {}", serial);
		println!("\tRefresh: {}", refresh);
		println!("\tRetry: {}", retry);
		println!("\tExpire: {}", expire);
		println!("\tMinimum: {}", minimum);

	}

	fn u8_to_u32(&mut self, idx: uint) -> u32 {

		let mut data: u32 = 0;

		data = data + (self.label[idx][0] as u32);
		data = data << 8;
		data = data + (self.label[idx][1] as u32);
		data = data << 8;
		data = data + (self.label[idx][2] as u32);
		data = data << 8;
		data = data + (self.label[idx][3] as u32);

		return data;
	}

	pub fn print_as_ipv4(&mut self) {
		print!("Address: ");
		for i in range(0, self.label.len()) {
			print!("{}", self.label[i][0]);
			if i<(self.label.len()-1) {
				print!(".");
			} else {
				println!("");
			}
		}
	}
	pub fn print_as_ipv6(&mut self) {
		print!("Address: ");
		for i in range(0, self.label.len()) {
			let mut hex: u16 = 0;
			for j in range(0, self.label[i].len()) {
				if j ==  0 {
					hex = (self.label[i][j] as u16) << 8;
				} else {
					hex = hex + (self.label[i][j] as u16);
				}
			}
			print!("{:X}", hex);
			if i<(self.label.len()-1) {
				print!(":");
			} else {
				println!("");
			}
		}
	}

	pub fn tld(&mut self) -> Option<Data> {
		if self.label.len() > 1 {
			let mut tld: Data = Data::new();
			tld.label.push(self.label[self.label.len()-1].clone());
			tld.length.push(self.length[self.label.len()-1].clone());
			return Some(tld);
		} else {
			return None;
		}
	}

	pub fn equals(&mut self, data: Data) -> bool {
		if self.label.len() != data.label.len() {
			return false;
		}
		for i in range(0u, self.label.len()) {
			if self.label[i].len() != data.label[i].len() {
				return false;
			}
			for j in range(0u, self.label[i].len()) {
				if data.label[i][j] != self.label[i][j] {
					return false;
				}
			}
		}
		return true;
	}
}