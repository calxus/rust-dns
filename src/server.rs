extern crate time;
extern crate term;

use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::collections::HashMap;
use std::num::SignedInt;
//use std::io::prelude::*;

use message::Message;
use resource::Resource;
use question::Question;
use data::Data;

#[deriving(Clone)]
pub struct Server {
	pub socket: UdpSocket,
	pub ip_lookup: HashMap<u16,SocketAddr>,
	pub msg_lookup: HashMap<u16,Message>,
	pub waiting_queue: Vec<Message>,
	pub cache: HashMap<Vec<u8>,Resource>,
	pub soa_cache: HashMap<Vec<u8>,Resource>,
	pub option: int
}

impl Server {
	pub fn new(addr: Vec<u8>, opt: int) -> Server {
    	let mut t = term::stdout().unwrap();
		let mut sock = SocketAddr {ip: Ipv4Addr(addr[0], addr[1], addr[2], addr[3]), port: 53};
		write!(t, "\nServer starting on: ");
    	t.fg(term::color::RED).unwrap();
    	(write!(t, "{}.{}.{}.{}\n", addr[0], addr[1], addr[2], addr[3])).unwrap();
    	t.reset();
		return Server {
			ip_lookup: HashMap::new(),
			msg_lookup: HashMap::new(),
			waiting_queue: vec![],
			socket: match UdpSocket::bind(sock) {
				Ok(s) 	=> s,
				Err(e) 	=> {panic!(e)},
			},
			cache: HashMap::new(),
			soa_cache: HashMap::new(),
			option: opt
		};
	}

	pub fn run(&mut self) {
		println!("Running...\n");
		loop {
			let mut buffer = [0, ..512];
			match self.socket.recv_from(&mut buffer) {
				Ok((length, src)) => {
					self.process(&mut buffer, length, src);
				},
				Err(e) => {
					println!("{}", e);
				},
			}
		}
	}

	pub fn process(&mut self, buffer: &mut [u8], length: uint, src: SocketAddr) {

		self.update_waiting_queue();

		let mut message = Message::new();
		match message.read_in(buffer, length) {
			Ok(()) => {},
			Err(a) => {
				println!("{}", a);
				return;
			}
		}

		match self.option {
			1 => {
				message.print();
			},
			2 => {
				message.questions[0].qname.print_as_hostname();
			},
			_ => {}
		}

		for i in range(0, self.waiting_queue.len()) {
			if self.waiting_queue[i].header.id == message.header.id {
				self.waiting_queue.remove(i);
				break;
			}
		}
		match self.check_cache(&mut message) {
			Some(mut res) => {
				match message.questions[0].qname.tld() {
					Some(tld) => {
						let mut i = 0;
						while i < res.len() {
							if res[i].rname.equals(tld.clone()) {
								match res[i].ip_addr() {
									Some(ip) => {
										self.send_message(&mut message, ip);
										return;
									}
									None => {}
								}
							}
							i = i + 1;
						}
					}
					None => {}
				}
				message.header.qr = 0x8000;
				message.header.ancount = res.len() as u16;
				message.answers.push_all(res.as_slice());
				self.send_message(&mut message, src);
			},
			None => {
				self.message_direction(&mut message, src);
			},
		}

		return;		
	}

	fn update_waiting_queue(&mut self) {
		let mut i = 0;
		while i < self.waiting_queue.len() {
			if ((time::precise_time_ns() as i64) - self.waiting_queue[i].timestamp).abs() > (800000000) {
				let mut msg = self.waiting_queue[i].clone();
				let sock = self.waiting_queue[i].next_server();
				match sock {
					Some(addr) => {
						self.send_message(&mut msg, addr);
						self.waiting_queue[i].timestamp = time::precise_time_ns() as i64;
						i = i + 1;
					},
					None => {
						self.waiting_queue.remove(i);
					}
				}
			} else {
				i = i + 1;
			}
		}
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
		match (message.header.ancount, message.header.nscount, message.header.arcount) {
	///		ancount 	nscount 	arcount 		action
			(0,			0,			0) 			=> {return},
			(1,			_,			_) 			=> {self.process_single_answer(message)},
			(2...999,	_,			_) 			=> {self.process_multiple_answers(message)},
			(0,			1...999,	0) 			=> {self.process_name_servers(message)},
			(0,			_,			1...999) 	=> {self.process_additional_records(message)},
			(_,			_,			_)			=> {return;}
		}
	}

	fn request(&mut self, message: &mut Message, src: SocketAddr) {
		self.ip_lookup.insert(message.header.id, src);
		self.send_message(message, SocketAddr {ip: Ipv4Addr(198, 41, 0, 4), port: 53});
		message.timestamp = time::precise_time_ns() as i64;
		self.waiting_queue.push(message.clone());
    	return;
	}

	fn process_single_answer(&mut self, message: &mut Message) {
		self.update_cache(message);
		if (message.contains_type(5)) && (message.header.ancount == 1) {
			let mut ns_query = Message::new();
			ns_query.generate_query(message.answers[0].rdata.write());
			match self.check_cache(&mut ns_query) {
				Some(res) => {
					message.answers.push_all(res.as_slice());
					message.header.ancount = res.len() as u16;
					match self.query_origin_addr(message) {
						Some(addr) => {
							self.send_message(message, addr);
							return;
						},
						None => {
							return;
						},
					}
				},
				None => {},
			}
			self.msg_lookup.insert(ns_query.header.id, message.clone());
			self.send_message(&mut ns_query, SocketAddr {ip: Ipv4Addr(198, 41, 0, 4), port: 53});
			ns_query.timestamp = time::precise_time_ns() as i64;
			self.waiting_queue.push(ns_query.clone());
			return;
		}

		while self.is_server_query_server(message) == true {
			match self.server_query_response(message) {
				Some(mut msg) => {
					if msg.contains_type(5) {
						msg.header.qr = 0x8000;
						msg.answers.push_all(message.answers.as_slice());
						msg.header.ancount = msg.header.ancount + message.header.ancount;
						match self.query_origin_addr(&mut msg) {
							Some(addr) => {

								self.send_message(&mut msg, addr);
								return;
							},
							None => {
								*message = msg.clone();
								continue;
							},
						}
					} else {
						msg.header.qr = 0x0000;
						msg.drop_records();
						match message.answers[0].ip_addr() {
							Some(ip) => {
								self.send_message(&mut msg, ip);
							},
							None => {}
						}
						self.msg_lookup.remove(&message.header.id);
						msg.timestamp = time::precise_time_ns() as i64;
						self.waiting_queue.push(msg.clone());
						return;
					}
				},
				None 	=> {
					match self.query_origin_addr(message) {
						Some(a) => {
							self.send_message(message, a);
							return;
						},
						None 	=> {
							return;
						}
					}
					
				},
			}
		}
		match self.query_origin_addr(message) {
			Some(a) => {
				self.send_message(message, a);
				return;
			},
			None 	=> {
				return;
			}
		}
	}

	fn process_multiple_answers(&mut self, message: &mut Message) {
		self.update_cache(message);
		if message.contains_type(1) && message.contains_type(5) {
			match self.query_origin_addr(message) {
				Some(addr) => {
					self.send_message(message, addr);
					return;
				},
				None 	=> {return;}
			}
		}
		if message.contains_type(1) {
			match self.query_origin_addr(message) {
				Some(addr) => {
					self.send_message(message, addr);
					return;
				},
				None 	=> {return;}
			}
		}
	}

	fn process_additional_records(&mut self, message: &mut Message) {
		self.update_cache(message);
		match message.next_server() {
			Some(addr) 	=> {
				message.header.qr = 0x0000;
				self.send_message(message, addr);
				message.timestamp = time::precise_time_ns() as i64;
				self.waiting_queue.push(message.clone());
				return;
			},
			None 		=> {
				return;
			}
		}
	}

	fn process_name_servers(&mut self, message: &mut Message) {
		self.update_cache(message);
		if message.questions[0].qtype == 0x0006 {
			let sa: SocketAddr = self.query_origin_addr(message).unwrap();
			self.send_message(message, sa);
			return;
		}
		let mut ns_query = Message::new();
		ns_query.generate_query(message.authority[0].rdata.write());
		match self.check_cache(&mut ns_query) {
			Some(res) => {
				for r in res.iter() {
					match r.clone().ip_addr() {
						Some(ip) => {
							message.header.qr = 0x0000;
							self.send_message(message, ip);
							return;
						}
						None => {}
					}
				}
			},
			None => {},
		}
		self.msg_lookup.insert(ns_query.header.id, message.clone());
		self.send_message(&mut ns_query, SocketAddr {ip: Ipv4Addr(198, 41, 0, 4), port: 53});
		ns_query.timestamp = time::precise_time_ns() as i64;
		self.waiting_queue.push(ns_query.clone());
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

	fn check_cache(&mut self, message: &mut Message) -> Option<Vec<Resource>> {

		if message.header.qdcount > 0 {

			let mut name = Data::new();
			name = message.questions[0].qname.clone();
			

			match self.retrieve_from_cache(&mut name) {
				Some(res) => {
						return Some(res);
				}
				None => {
					name = match message.questions[0].qname.tld().clone() {
						Some(n) => {n}
						None => {Data::new()}
					};
					match self.retrieve_from_cache(&mut name) {
						Some(res) => {
							return Some(res);
						}
						None => {
							return None;
						}
					}
				}
			}
			

		} else {
			return None;
		}
	}

	fn retrieve_from_cache(&mut self, name: &mut Data) -> Option<Vec<Resource>> {
		let mut res: Vec<Resource> = vec![];
		while self.cache.contains_key(&name.write()) == true {
			let mut temp_res = self.cache[name.write()].clone();
			let mut num: i64 = (temp_res.ttl as i64) * 1000000000;
			if (num + temp_res.cache_timeout) < time::precise_time_ns() as i64 {
				self.cache.remove(&name.write());
				return Some(res);
			} else {
				res.push(temp_res.clone());
				*name = temp_res.rdata.clone();
			}
		}
		if res.len() > 0 {
			return Some(res);
		} else {
			return None;
		}
	}

	fn update_cache(&mut self, message: &mut Message) {
		for i in range(0, message.header.ancount as uint) {
			match self.cache.contains_key(&message.answers[i].rname.write()) {
				true => {
					continue;
				},
				false => {
					message.answers[i].cache_timeout = time::precise_time_ns() as i64;
					self.cache.insert(message.answers[i].rname.write(), message.answers[i].clone());
				}
			}
		}

		for i in range(0, message.header.arcount as uint) {
			match self.cache.contains_key(&message.additional[i].rname.write()) {
				true => {
					continue;
				},
				false => {
					/*
					if message.header.nscount > 0 {
						message.additional[i].rname = message.authority[0].r.clone();
					}
					*/
					message.additional[i].cache_timeout = time::precise_time_ns() as i64;
					self.cache.insert(message.additional[i].rname.write(), message.additional[i].clone());
				}
			}
		}
	}

	fn query_origin_addr(&mut self, message: &mut Message) -> Option<SocketAddr> {
		match self.ip_lookup.contains_key(&message.header.id) {
			true 	=> {
				let addr = self.ip_lookup[message.header.id].clone();
				self.ip_lookup.remove(&message.header.id);
				Some(addr)
			},
			false 	=> {
				None
			},
		}
	}

	fn server_query_response(&mut self, message: &mut Message) -> Option<Message> {
		match self.msg_lookup.contains_key(&message.header.id) {
			true 	=> {
				let mut msg = self.msg_lookup[message.header.id].clone();
				self.msg_lookup.remove(&message.header.id);
				Some(msg)
			},
			false	=> {
				None
			}
		}
	}

	fn is_server_query_server(&mut self, message: &mut Message) -> bool {
		match self.msg_lookup.contains_key(&message.header.id) {
			true 	=> {
				return true;
			},
			false	=> {
				return false;
			}
		}
	}
}