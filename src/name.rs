/*
Gordon Adam
1107425

Struct that represents a name

* The plan is to change this so it also handles rdata to simplify code *
* Also merge the two functions that add data to the struct *
*/	

use std::default;
use std::io::BufReader;


#[deriving(Default,Clone)]
pub struct Name {
    pub label: Vec<String>, // This is the seperate strings of the host name eg. ["www", "google", "com"]
    pub length: Vec<u8>, // This is the vector of lengths of the labels eg. ["3", "6", "3"]
    pub pointer: u16 // This is only here temporarily and represents a pointer
}

impl Name {

	// Creates an instance of the struct with default values
	pub fn new() -> Name {
		return Name {..default::Default::default()};
	}

	// takes a buffer reader to add values to the struct
	pub fn read_in(&mut self, reader: &mut BufReader) {

		let mut byte_buffer = reader.read_u8().unwrap();

		if(byte_buffer & 0xc0) == 0xc0 {
			self.pointer = (0xc0 * 0x100);
			self.pointer = self.pointer + 0x000c;
			reader.read_u8();
			return;
		}

    	while (byte_buffer != 0) {

        	let mut label: Vec<u8> = vec![];

        	for i in range(0, byte_buffer)  {
            	let mut temp_byte_buffer = reader.read_u8().unwrap();
            	label.push(temp_byte_buffer);
        	}

        	self.label.push(String::from_utf8(label).unwrap());
        	self.length.push(byte_buffer);

        	byte_buffer = reader.read_u8().unwrap();
    	}
	}

	// takes a vector of 8-bit characters to add values to the struct
	pub fn read_fr_vec(&mut self, name: Vec<u8>) {
		let mut idx = 0;
		let mut byte_buffer = name[idx];

		while (byte_buffer != 0) {
			let mut label: Vec<u8> = vec![];

			idx = idx + 1;

			for i in range(0, byte_buffer.to_uint().unwrap()) {
				label.push(name[idx + i]);
			}

			self.label.push(String::from_utf8(label).unwrap());
			self.length.push(byte_buffer);

			idx = idx + byte_buffer.to_uint().unwrap();
			byte_buffer = name[idx];
		}
	}

	// function that will print a label out in the format "www.google.com"
	pub fn print(&mut self) {
		print!("\tName: ");
		for i in range(0, self.label.len()) {
			print!("{}", self.label[i]);
			if(i<(self.label.len()-1)) {
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
}