/*
Gordon Adam
1107425

Struct to represent a Resource Record
*/

use std::default;
use std::io::BufReader;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use data;

#[deriving(Default,Clone)]
pub struct Resource {
    pub rname: data::Data, // This is the name the resource record pertains to
    pub rtype: u16, // This is the type of the resource record
    pub rclass: u16, // This is the class of the resource record
    pub ttl: u32, // This is the time to live of the resource record
    pub rdlength: u16, // This is the length of the data held in rdata
    pub rdata: data::Data, // this is the data of the resource record

    pub cache_timeout: i64,
}

impl Resource {

	// Creates an instance of the struct with default values
	pub fn new() -> Resource {
		return Resource {..default::Default::default()};
	}

	// This reads in from a buffered reader the values into the struct
	pub fn read_in(&mut self, reader: &mut BufReader, msg_copy: &mut Vec<u8>) -> Result<(), String> {
		self.rname = data::Data::new();
    	self.rname.read_hostname(reader, msg_copy);

    	self.rtype = reader.read_be_u16().unwrap();
    	self.rclass = reader.read_be_u16().unwrap();
    	self.ttl = reader.read_be_u32().unwrap();
    	self.rdlength = reader.read_be_u16().unwrap();
    	
    	match self.rtype {
    		0x0001 => {self.rdata.read_ipv4_addr(reader)},
    		0x0002 => {self.rdata.read_hostname(reader, msg_copy)},
    		0x0005 => {self.rdata.read_hostname(reader, msg_copy)},
    		0x0006 => {self.rdata.read_soa(reader, msg_copy)},
    		0x000c => {return Err("Error: Resource Type (12) not supported".to_string());},
    		0x000f => {return Err("Error: Resource Type (15) not supported".to_string());},
    		0x0010 => {return Err("Error: Resource Type (16) not supported".to_string());},
    		0x0011 => {return Err("Error: Resource Type (17) not supported".to_string());},
    		0x0012 => {return Err("Error: Resource Type (18) not supported".to_string());},
    		0x0018 => {return Err("Error: Resource Type (24) not supported".to_string());},
    		0x0019 => {return Err("Error: Resource Type (25) not supported".to_string());},
    		0x001c => {self.rdata.read_ipv6_addr(reader)},
    		_ => {return Err("Error: Resource Type (?) not supported".to_string());},
    	}
    	if (self.rtype == 2) || (self.rtype == 5) {
    		self.rdlength = 0;
    		for i in range(0u, self.rdata.length.len()) {
    			self.rdlength = self.rdlength + (self.rdata.length[i] as u16 + 1);
    		}
    		self.rdlength = self.rdlength + 1;
    	}

    	self.cache_timeout = 0;

    	return Ok(());
	}

	

	// converts the struct to a vector of u8 characters and returns it
	pub fn write(&mut self) -> Vec<u8>{
		let mut resource_buffer: Vec<u8> = vec![];

   		resource_buffer.push_all(self.rname.write().as_slice());

    	split_u16(self.rtype, &mut resource_buffer);
    	split_u16(self.rclass, &mut resource_buffer);

    	let mut temp_buffer: u16;

    	temp_buffer = ((self.ttl & 0xFFFF0000) >> 16) as u16;
    	split_u16(temp_buffer, &mut resource_buffer);
    	temp_buffer = ((self.ttl & 0x0000FFFF) >> 0) as u16;
    	split_u16(temp_buffer, &mut resource_buffer);

    	split_u16(self.rdlength, &mut resource_buffer);
    	if self.rtype == 1 {
    		resource_buffer.push_all(self.rdata.write_ip_addr().as_slice());
    	} else if self.rtype == 5 {
    		resource_buffer.push_all(self.rdata.write_cname().as_slice());
    	} else {
    		resource_buffer.push_all(self.rdata.write().as_slice());
    	}
    	return resource_buffer;
	}

	// returns the ip address contained within rdata of the resource record
	pub fn ip_addr(&mut self) -> Option<SocketAddr> {
		if self.rtype == 0x0001 {
			let ip_addr = self.rdata.get_ipv4_addr();
			let sock = SocketAddr {ip: Ipv4Addr(ip_addr[0], ip_addr[1], ip_addr[2], ip_addr[3]), port: 53};
			return Some(sock);
		} else {
			return None
		}
	}


	pub fn print(&mut self) {
		print!("\t");
		self.rname.print_as_hostname();
		print!("\t");
		match self.rtype & 0xFFFF {
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
		match self.rclass & 0xFFFF {
			0x0001 => {println!("Class: IN (0x0001)")},
			0x0002 => {println!("Class: CS (0x0002)")},
			0x0003 => {println!("Class: CH (0x0003)")},
			0x0004 => {println!("Class: HS (0x0004)")},
			_ => {println!("somethings wrong")}
		}
		print!("\t");
		println!("Time to live: {}", self.ttl);
		print!("\t");
		println!("Data Length: {}", self.rdlength);
		print!("\t");
		match self.rtype {
			0x0001 => {self.rdata.print_as_ipv4()},
			0x0002 => {self.rdata.print_as_hostname()},
			0x0005 => {self.rdata.print_as_hostname()},
			0x0006 => {self.rdata.print_as_soa()},
			0x000c => {return;},
			0x000f => {return;},
    		0x0010 => {return;},
    		0x0011 => {return;},
    		0x0012 => {return;},
    		0x0018 => {return;},
    		0x0019 => {return;},
			0x001c => {self.rdata.print_as_ipv6()},
			_ => {panic!("This has not been covered")},
		}
		println!("");
	}
}

// Splits a u16 character into two u8 characters and pushes them to a buffer provided
pub fn split_u16(u: u16, message_buffer: &mut Vec<u8>) {
   	message_buffer.push(((u & 0xFF00) >> 8) as u8);
    message_buffer.push(((u & 0x00FF) >> 0) as u8);
}