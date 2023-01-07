//! # Reply
//! Module for server replies/reponses

use std::net::IpAddr;
use crate::SOCKS_VERSION;
use crate::request::addr_type::*;

#[allow(missing_docs)]
pub mod reply_opt {
    pub const SUCCEEDED: u8 = 0x0;
    pub const SOCKS_SERVER_FAILURE: u8 = 0x1;
    pub const CONNECTION_NOT_ALLOWED: u8 = 0x2;
    pub const NETWORK_UNREACHABLE: u8 = 0x3;
    pub const HOST_UNREACHABLE: u8 = 0x4;
    pub const CONNECTION_REFUSED: u8 = 0x5;
    pub const TTL_EXPIRED: u8 = 0x6;
    pub const COMMAND_NOT_SUPPORTED: u8 = 0x7;
    pub const ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x8;
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub fn new(rep: u8, atyp: u8, bnd_addr: IpAddr, bnd_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            rep: rep as u8,
            rsv: 0x0,
            atyp: atyp as u8,
            bnd_addr,
            bnd_port
        }
    }

    pub fn reply(&self) -> u8 {
        self.rep
    }

    pub fn addr_type(&self) -> u8 {
        self.atyp
    }

    pub fn socket_addr(&self) -> ( IpAddr, u16 ) {
        (self.bnd_addr, self.bnd_port)
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