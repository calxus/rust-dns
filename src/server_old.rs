extern crate time;

use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::collections::HashMap;
use std::num::SignedInt;

use message::Message;
use question::Question;

#[deriving(Clone)]
pub struct Server {
	pub ip_lookup: HashMap<u16,SocketAddr>,
	pub msg_lookup: HashMap<u16,Message>,
	pub waiting_resp: Vec<Message>,
	pub socket: UdpSocket,
}

impl Server {
	pub fn new() -> Server {
		let addr = SocketAddr {ip: Ipv4Addr(172, 30, 159, 85), port: 53};
		return Server {
			ip_lookup: HashMap::new(),
			msg_lookup: HashMap::new(),
			waiting_resp: vec![],
			socket: match UdpSocket::bind(addr) {
				Ok(s) 	=> s,
				Err(e) 	=> {panic!(e)},
			}
		};
	}

	pub fn run(&mut self) {
		loop {
    		let mut buffer = [0, ..512];
			match self.socket.recv_from(&mut buffer) {
				Ok((length, src)) => {
					self.handle_message(&mut buffer, length, src);
				},
				Err(e) => {
					println!("{}", e);
				},
			}
			let mut i = 0;
			//println!("{}", self.waiting_resp.len());
			while i < self.waiting_resp.len() {
				if ((time::precise_time_ns() as i64) - self.waiting_resp[i].timestamp).abs() > (5000000000) {
					let mut msg = self.waiting_resp[i].clone();
					let sock = self.waiting_resp[i].next_server();
					match sock {
						Ok(o) => {
							self.send_message(&mut msg, o);
							self.waiting_resp[i].timestamp = time::precise_time_ns() as i64;
							i = i + 1;
						},
						Err(e) => {
							self.waiting_resp.remove(i);
						},
					}
				} else {
					i = i + 1;
				}
			}
		}
	}

	fn handle_message(&mut self, buffer: &mut [u8], length: uint, src: SocketAddr) {
		let mut message = Message::new();
		match message.read_in(buffer, length) {
			Ok(()) => {},
			Err(a) => {
				println!("{}", a);
				return;
			}
		}

		for i in range(0, self.waiting_resp.len()) {
			if self.waiting_resp[i].header.id == message.header.id {
				self.waiting_resp.remove(i);
				break;
			}
		}


		match message.questions[0].qtype {
			0x0001 	=> {self.message_direction(&mut message, src);},
			0x0002 	=> {self.message_direction(&mut message, src);},
			0x001c 	=> {self.message_direction(&mut message, src);},
			_		=> {return;},
		}
		return;		
	}

	fn message_direction(&mut self, message: &mut Message, src: SocketAddr) {
		match message.header.qr {
			0x0000 	=> {self.request(message, src);},
			0x8000 	=> {self.response(message);},
			_		=> {return;},
		}
		return;
	}

	fn response(&mut self, message: &mut Message) {
		if message.header.ancount > 0 {
			for i in range(0u, message.header.ancount as uint) {
				if message.answers[i].rtype == 1 {
					self.response_answer(message, i);
					return;
				}
			}
			for i in range(0u, message.header.ancount as uint) {
				if message.answers[i].rtype == 5 {
					self.response_cname(message, i);
					return;
				}
			}
		} else if message.header.arcount > 0 {
			self.response_additional_record(message);
		} else if message.header.nscount > 0 {
			self.response_name_server(message);
		} else {
			return;
		}
	}

	fn request(&mut self, message: &mut Message, src: SocketAddr) {
		self.ip_lookup.insert(message.header.id, src);
		self.send_message(message, SocketAddr {ip: Ipv4Addr(198, 41, 0, 4), port: 53});
		message.timestamp = time::precise_time_ns() as i64;
		self.waiting_resp.push(message.clone());
    	return;
	}

	fn response_answer(&mut self, message: &mut Message, idx: uint) {
		match self.msg_lookup.contains_key(&message.header.id) {
			true 	=> {
				let mut msg = self.msg_lookup[message.header.id].clone();
				for i in range(0u, msg.header.ancount as uint) {
					if msg.answers[i].rtype == 5 {
						msg.header.qr = 0x8000;
						message.answers[0].rname = msg.questions[0].qname.clone();
						msg.answers.push(message.answers[0].clone());
						msg.header.ancount = msg.header.ancount + 1;
						match self.ip_lookup.contains_key(&msg.header.id) {
							true 	=> {
								let sock = self.ip_lookup[msg.header.id].clone();
								self.send_message(&mut msg, sock);
								self.ip_lookup.remove(&msg.header.id);
								return;
							},
							false 	=> {
								return;
							},
						}	
						return;
					}
				}
				msg.header.qr = 0x0000;
				msg.drop_records();
				self.send_message(&mut msg, message.answers[idx].ip_addr());
				self.msg_lookup.remove(&message.header.id);
				msg.timestamp = time::precise_time_ns() as i64;
				self.waiting_resp.push(msg.clone());
				return;
			},
			false 	=> {
				match self.ip_lookup.contains_key(&message.header.id) {
					true 	=> {
						let sock = self.ip_lookup[message.header.id].clone();
						self.send_message(message, sock);
						self.ip_lookup.remove(&message.header.id);
						return;
					},
					false 	=> {
						return;
					},
				}
					
			},
		}
	}

	fn response_additional_record(&mut self, message: &mut Message) {
		let sock = message.next_server();
		if sock.is_err() {
			println!("NO");
			return;
		}
		message.drop_records();
		message.header.qr = 0x0000;
		self.send_message(message, sock.unwrap());
		message.timestamp = time::precise_time_ns() as i64;
		self.waiting_resp.push(message.clone());
		return;
	}

	fn response_cname(&mut self, message: &mut Message, idx: uint) {
		let mut cname_query = Message::new();
		cname_query.generate_query(message.answers[idx].rdata.write());
		self.msg_lookup.insert(cname_query.header.id, message.clone());
		self.send_message(&mut cname_query, SocketAddr {ip: Ipv4Addr(198, 41, 0, 4), port: 53});
		cname_query.timestamp = time::precise_time_ns() as i64;
		self.waiting_resp.push(cname_query.clone());
		return;
	}

	fn response_name_server(&mut self, message: &mut Message) {
		let mut ns_query = Message::new();
		ns_query.generate_query(message.authority[0].rdata.write());
		self.msg_lookup.insert(ns_query.header.id, message.clone());
		self.send_message(&mut ns_query, SocketAddr {ip: Ipv4Addr(198, 41, 0, 4), port: 53});
		ns_query.timestamp = time::precise_time_ns() as i64;
		self.waiting_resp.push(ns_query.clone());
		return;
	}

	fn send_message<'a>(&'a mut self, message: &'a mut Message, sock: SocketAddr) -> &mut Message {
		match self.socket.send_to(message.clone().write().as_slice(), sock) {
        	Ok(()) => {
        		return message;
        	},
        	Err(e) => {
        		println!("{}, failed to send", e);
        		return message;
        	},
    	};
	}
}