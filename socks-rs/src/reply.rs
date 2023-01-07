//! # Reply
//! Module for server replies/reponses

use crate::request::addr_type;
use crate::{utils::Sendible, SOCKS_VERSION};

#[allow(missing_docs, unused)]
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
pub struct Reply<'a> {
    version: u8,
    rep: u8,
    rsv: u8,
    atyp: u8,
    bnd_addr: &'a [u8],
    bnd_port: u16,
}

#[allow(unused)]
impl<'a> Reply<'a> {
    pub fn new(rep: u8, atyp: u8, bnd_addr: &'a [u8], bnd_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            rep,
            rsv: 0x0,
            atyp,
            bnd_addr,
            bnd_port,
        }
    }

    pub fn reply(&self) -> u8 {
        self.rep
    }

    pub fn addr_type(&self) -> u8 {
        self.atyp
    }

    pub fn socket_addr(&self) -> (&[u8], u16) {
        (self.bnd_addr, self.bnd_port)
    }
}

impl<'s> Sendible<'s> for Reply<'s> {
    fn serialize(&self) -> Option<Vec<u8>> {
        let mut data = vec![self.version, self.rep, self.rsv, self.atyp];
        data.extend(self.bnd_addr);
        data.extend([
            ((self.bnd_port >> 8) & 0xff) as u8,
            (self.bnd_port & 0xff) as u8,
        ]);
        Some(data)
    }

    fn deserialize(data: &'s [u8]) -> Option<Self> {
        let (version, rep, rsv, atyp) = (data[0], data[1], data[2], data[3]);

        let offset = match atyp {
            addr_type::IP_V4 => 8_usize,
            addr_type::DOMAIN_NAME => panic!("Can't do DOMAINNAME yet"),
            addr_type::IP_V6 => 20_usize,
            _ => panic!("Invalid address type"),
        };

        let bnd_addr = &data[4..offset];
        println!(">> {bnd_addr:?}");
        let bnd_port = &data[offset..];
        let bnd_port = (bnd_port[0] as u16) << 8 | (bnd_port[1] as u16);

        Some(Self {
            version,
            rep,
            rsv,
            atyp,
            bnd_addr,
            bnd_port,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reply_serr_deser() {
        let reply = Reply::new(
            reply_opt::COMMAND_NOT_SUPPORTED,
            addr_type::IP_V4,
            &[127, 0, 0, 1],
            1080,
        );
        let serialize = reply.serialize().unwrap();
        let new = Reply::deserialize(&serialize).unwrap();

        println!("{serialize:?}");
        println!("{reply:?}\n{new:?}");

        assert_eq!(reply, new)
    }
}
