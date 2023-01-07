//! # Requests
//! Contains the `Request` struct according and 
//! to [`RFC 1928`](https://datatracker.ietf.org/doc/html/rfc1928)

use std::net::IpAddr;
use crate::SOCKS_VERSION;

#[allow(missing_docs)]
pub mod addr_type {
    pub const IP_V4: u8 = 0x1;
    pub const DOMAIN_NAME: u8 = 0x3;
    pub const IP_V6: u8 = 0x4;
}

#[allow(missing_docs)]
pub mod command {
    pub const CONNECT: u8 = 0x1;
    pub const BIND: u8 = 0x2;
    pub const UDP_ASSOCIATE: u8 = 0x3;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Request {
    version: u8,
    cmd: u8,
    rsv: u8, // reserved, always 0x0
    atyp: u8,
    dst_addr: IpAddr,
    dst_port: u16
}

#[allow(unused)]
impl Request {
    pub fn new(cmd: u8, atyp: u8, dst_addr: IpAddr, dst_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            cmd,
            rsv: 0x0,
            atyp,
            dst_addr,
            dst_port
        }
    }

    pub fn command(&self) -> u8 {
        self.cmd
    }

    pub fn addr_type(&self) -> u8 {
        self.atyp
    }

    pub fn socket_addr(&self) -> ( IpAddr, u16 ) {
        (self.dst_addr, self.dst_port)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};
    use crate::utils::Sendible;

    #[test]
    fn request_serr_deser() {
        let request = Request::new(Command::Connect, AddrType::IpV4, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1080);
        let serialized = request.serialize().unwrap();
        let new = Request::deserialize(&serialized).unwrap();
        
        println!("{serialized:?}");
        println!("{request:?}\n{new:?}");

        assert_eq!(request, new);
    }
}