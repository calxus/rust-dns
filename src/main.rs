#![feature(slicing_syntax)]

use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
fn main() {
    let addr = SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: 53 };
    let mut socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => fail!("couldn't bind socket: {}", e),
    };

    let mut buf = [0, ..1024];
    match socket.recv_from(buf) {
        Ok(T) => {println!("{}", T.to_string())},
        Err(e) => println!("couldn't receive a datagram: {}", e)
    }
    drop(socket); // close the socket
}