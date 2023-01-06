//! # Requests
//! Contains the `Request` struct according and 
//! to [`RFC 1928`](https://datatracker.ietf.org/doc/html/rfc1928)

use std::net::IpAddr;
use crate::SOCKS_VERSION;

#[allow(missing_docs)]
#[repr(u8)]
#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub enum AddrType {
    IpV4 = 0x1,
    DomainName = 0x3,
    IpV6 = 0x4,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub enum Command {
    Connect = 0x1,
    Bind = 0x2,
    UdpAssociate = 0x3,
}

#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub struct Request {
    version: u8,
    cmd: Command,
    rsv: u8, // reserved, always 0x0
    atyp: AddrType,
    dst_addr: IpAddr,
    dst_port: u16
}

#[allow(unused)]
impl Request {
    pub fn new(cmd: Command, atyp: AddrType, dst_addr: IpAddr, dst_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            cmd,
            rsv: 0x0,
            atyp,
            dst_addr,
            dst_port
        }
    }

    pub fn command(&self) -> &Command {
        &self.cmd
    }

    pub fn addr_type(&self) -> &AddrType {
        &self.atyp
    }

    pub fn socket_addr(&self) -> ( IpAddr, u16 ) {
        (self.dst_addr, self.dst_port)
    }
}

impl From<u8> for AddrType {
    fn from(value: u8) -> Self {
        match value {
            0x1 => AddrType::IpV4,
            0x3 => AddrType::DomainName,
            0x4 => AddrType::IpV6,
            _ => panic!("Out of range value")
        }
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