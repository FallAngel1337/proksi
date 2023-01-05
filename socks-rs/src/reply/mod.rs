//! # Reply
//! Module for server replies/reponses

mod reply_opts;

pub use reply_opts::ReplyOpt;
use crate::SOCKS_VERSION;
use crate::addr_type::AddrType;
use crate::utils::Sendible;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Reply {
    version: u8,
    rep: ReplyOpt,
    rsv: u8,
    atyp: AddrType,
    bnd_addr: Vec<u8>,
    bnd_port: u16,
}

impl Reply {
    pub fn new(rep: ReplyOpt, atyp: AddrType, bnd_addr: &[u8], bnd_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            rep,
            rsv: 0x0,
            atyp,
            bnd_addr: bnd_addr.to_vec(),
            bnd_port
        }
    }

    pub fn reply(&self) -> &ReplyOpt {
        &self.rep
    }

    pub fn addr_type(&self) -> &AddrType {
        &self.atyp
    }

    pub fn socket_addr(&self) -> ( &[u8], u16 ) {
        (&self.bnd_addr, self.bnd_port)
    }
}

impl<'s> Sendible<'s> for Reply {}