use std::net::SocketAddr;

use tokio::net::UdpSocket;

use crate::Span;

pub trait Transport {
    fn append(&mut self, span: Span);
    fn flush(&mut self);
}

pub struct UdpTransport {
    socket: UdpSocket,
    remote_addr: SocketAddr,
}

impl UdpTransport {
    pub fn new(remote_addr: SocketAddr) -> Self {
        let local_addr: SocketAddr = if remote_addr.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        }
        .parse()
        .unwrap();
        let socket = UdpSocket::bind(&local_addr).expect("failed to bind UDP addr");

        Self {
            socket,
            remote_addr,
        }
    }

    pub fn send(&mut self, data: &[u8]) {
        self.socket.send_to(data, &self.remote_addr);
    }
}

impl Transport for UdpTransport {
    fn append(&mut self, span: Span) {}
    fn flush(&mut self) {}
}
