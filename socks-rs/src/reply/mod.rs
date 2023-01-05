//! # Reply
//! Module for server replies/reponses

mod reply_opts;


use std::net::IpAddr;
pub use reply_opts::ReplyOpt;
use crate::SOCKS_VERSION;
use crate::addr_type::AddrType;
use crate::utils::Sendible;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct Reply {
    version: u8,
    rep: ReplyOpt,
    rsv: u8,
    atyp: AddrType,
    bnd_addr: IpAddr,
    bnd_port: u16,
}

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

impl<'s> Sendible<'s> for Reply {}