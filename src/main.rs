/*
Gordon Adam
1107425

This is the main program though as it is getting over complex the plan is to change it to the server program
and also break it up
*/

#![feature(macro_rules)]

use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::collections::HashMap;
use std::num::SignedInt;
use std::io::Command;
use std::string::String;
use std::io::BufReader;
use std::str;

use message::Message;

mod server;
mod header;
mod question;
mod data;
mod resource;
mod message;

fn main() {
	let mut a: Vec<u8> = vec![(127 as u8), (0 as u8), (0 as u8), (1 as u8)];
	let mut addr: Vec<u8> = vec![];

	let output = match Command::new("ifconfig").output() {
    	Ok(output) => output,
    	Err(e) => panic!("failed to execute process: {}", e),
	};

	let mut reader = BufReader::new(output.output.as_slice());
	let mut found: bool = false;	
	
	while found == false {
		let mut line = reader.read_line().unwrap().into_bytes();
		let mut buffer: Vec<u8> = vec![];
		let mut i: uint = 0;
		for i in range(0u, 5) {
			buffer.push(line[i]);
		}
		if String::from_utf8(buffer.clone()).unwrap() == String::from_str("en0: ") {
			while found == false {
				buffer = vec![];
				line = reader.read_line().unwrap().into_bytes();
				for j in range(1u, 6) {
					buffer.push(line[j]);
				}
				if String::from_utf8(buffer.clone()).unwrap() == String::from_str("inet ") {
					buffer = vec![];
					let mut j: uint = 6;
					while line[j] != 0x0020 {
						if line[j] == 0x002e {
							j = j + 1;
							let mut num: u8 = from_str(str::from_utf8(buffer.as_slice()).unwrap()).unwrap();
							addr.push(num);
							buffer = vec![];
							continue;
						}
						buffer.push(line[j]);
						j = j + 1;
					}
					let mut num: u8 = from_str(str::from_utf8(buffer.as_slice()).unwrap()).unwrap();
					addr.push(num);
					buffer = vec![];
					break;
				}
			}
			break;
		}
		continue;
		found = true;
	}

	let mut s = server::Server::new(addr);

	s.run();
}

