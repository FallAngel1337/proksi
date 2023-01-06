//! # Reply
//! Module for server replies/reponses

use std::net::IpAddr;
use crate::SOCKS_VERSION;
use crate::request::AddrType;

#[allow(missing_docs)]
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


#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct Reply {
    version: u8,
    rep: ReplyOpt,
    rsv: u8,
    atyp: AddrType,
    bnd_addr: IpAddr,
    bnd_port: u16,
}

#[allow(unused)]
impl Reply {
    pub fn new(rep: ReplyOpt, atyp: AddrType, bnd_addr: IpAddr, bnd_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            rep,
            rsv: 0x0,
            atyp,
            bnd_addr,
            bnd_port
        }
    }

    pub fn reply(&self) -> &ReplyOpt {
        &self.rep
    }

    pub fn addr_type(&self) -> &AddrType {
        &self.atyp
    }

    pub fn socket_addr(&self) -> ( IpAddr, u16 ) {
        (self.bnd_addr, self.bnd_port)
    }
}