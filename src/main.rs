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
use std::os::args;

use message::Message;

mod server;
mod header;
mod question;
mod data;
mod resource;
mod message;

fn main() {
	print!("\x1b[2J\x1b[H");
	if args().len() != 3 {
		println!("Error: 2 arguments need to be provided");
		return;
	}
	let mut mess_arg: String = args()[1].clone();
	let mut addr_arg: String = args()[2].clone();

	println!("Rust DNS Resolver\n")

	let mut addr: Vec<u8> = vec![(0 as u8), (0 as u8), (0 as u8), (0 as u8)];

	if addr_arg.as_slice() == "detect" {
		match detect_addr() {
			Some(a) => {
				println!("IP Address succesfully detected");
				addr = a;
			},
			None => {
				println!("Error: IP Address detection not supported");
				println!("Listening on all interfaces");
			}
		}
	}
	
	match mess_arg.as_slice() {
		"none" => {
			println!("No message details will be printed");
			let mut s = server::Server::new(addr, 0);
			s.run();
		}
		"details" => {
			println!("Detailed message information will be printed");
			let mut s = server::Server::new(addr, 1);
			s.run();
    	},
		"hostname" => {
			println!("Simplified details will be printed");
			let mut s = server::Server::new(addr, 2);
			s.run();
		}
		_ => {
			println!("Error: argument not recognised");
			return;
		}
	}
}

pub fn detect_addr() -> Option<Vec<u8>> {
	let mut addr: Vec<u8> = vec![];

	let output = match Command::new("ifconfig").arg("en0").output() {
    	Ok(output) => output,
    	Err(e) => panic!("failed to execute process: {}", e),
	};

	if output.output.len() == 0 {
		return None;
	}

	let mut reader = BufReader::new(output.output.as_slice());
	let mut found: bool = false;	
	let mut line = reader.read_line().unwrap().into_bytes();
	let mut buffer: Vec<u8> = vec![];
	let mut i: uint = 0;
	while found == false {
		match reader.read_line() {
			Ok(l) => {
				line = l.into_bytes();
			},
			Err(e) => {return None;}
		}
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
			found = true;
		}
		buffer = vec![];
	}
	return Some(addr);
}

