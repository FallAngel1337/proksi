//! # Reply
//! Module for server replies/reponses

use std::net::IpAddr;
use crate::SOCKS_VERSION;
use crate::request::AddrType;

#[allow(missing_docs)]
#[repr(u8)]
#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub enum ReplyOpt {
    Succeeded = 0x0,
    SocksServerFailure = 0x1,
    ConnectionNotAllowed = 0x2,
    NetworkUnreachable = 0x3,
    HostUnreachable = 0x4,
    ConnectionRefused = 0x5,
    TtlExpired = 0x6,
    CommandNotSupported = 0x7,
    AddressTypeNotSupported = 0x8,
}

#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub struct Reply {
    version: u8,
    rep: u8,
    rsv: u8,
    atyp: u8,
    bnd_addr: IpAddr,
    bnd_port: u16,
}

#[allow(unused)]
impl Reply {
    pub fn new(rep: ReplyOpt, atyp: AddrType, bnd_addr: IpAddr, bnd_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            rep: rep as u8,
            rsv: 0x0,
            atyp: atyp as u8,
            bnd_addr,
            bnd_port
        }
    }

    pub fn reply(&self) -> ReplyOpt {
        ReplyOpt::from(self.rep)
    }

    pub fn addr_type(&self) -> AddrType {
        AddrType::from(self.atyp)
    }

    pub fn socket_addr(&self) -> ( IpAddr, u16 ) {
        (self.bnd_addr, self.bnd_port)
    }
}

impl From<u8> for ReplyOpt {
    fn from(value: u8) -> Self {
        match value {
            0x0 => ReplyOpt::Succeeded,
            0x1 => ReplyOpt::SocksServerFailure,
            0x2 => ReplyOpt::ConnectionNotAllowed,
            0x3 => ReplyOpt::NetworkUnreachable,
            0x4 => ReplyOpt::HostUnreachable,
            0x5 => ReplyOpt::ConnectionRefused,
            0x6 => ReplyOpt::TtlExpired,
            0x7 => ReplyOpt::CommandNotSupported,
            0x8 => ReplyOpt::AddressTypeNotSupported,
            _ => panic!("Out of range value")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::Sendible;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn reply_serr_deser() {
        let reply = Reply::new(ReplyOpt::CommandNotSupported, AddrType::IpV4, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 67);
        let serialize = reply.serialize().unwrap();
        let new = Reply::deserialize(&serialize).unwrap();
        
        println!("{serialize:?}");
        println!("{reply:?}\n{new:?}");

        assert_eq!(reply, new)
    }
}