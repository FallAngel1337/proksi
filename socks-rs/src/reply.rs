//! # Reply
//! Module for server replies/reponses

use crate::request::addr_type;
use crate::{Sendible, SOCKS_VERSION};

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

/// The reply response struct (server-only)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reply<'a> {
    /// protocol version (0x5)
    pub version: u8,

    /// reply field
    pub rep: u8,

    /// RESERVED
    pub rsv: u8,

    /// address type
    pub atyp: u8,

    /// server bound address
    pub bnd_addr: &'a [u8],

    /// server bound port in network octet order
    pub bnd_port: u16,
}

impl<'a> Reply<'a> {
    /// Creates a new reply response
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
}

impl<'s> Sendible<'s> for Reply<'s> {
    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        let mut data = vec![self.version, self.rep, self.rsv, self.atyp];
        
        if self.atyp == addr_type::DOMAIN_NAME {
            data.push(self.bnd_addr.len() as u8);
        }
        
        data.extend(self.bnd_addr);
        data.extend([
            ((self.bnd_port >> 8) & 0xff) as u8,
            (self.bnd_port & 0xff) as u8,
        ]);
        Ok(data)
    }

    fn deserialize(data: &'s [u8]) -> std::io::Result<Self> {
        let (version, rep, rsv, atyp) = (data[0], data[1], data[2], data[3]);

        let (start, offset) = match atyp {
            addr_type::IP_V4 => (4, 8),
            addr_type::DOMAIN_NAME => (5, 5 + data[4] as usize),
            addr_type::IP_V6 => (4, 20),
            atyp => return Err(
                std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    format!("Invalid address type {atyp}")
            )),
        };

        let bnd_addr = &data[start..offset];
        let bnd_port = &data[offset..];
        let bnd_port = (bnd_port[0] as u16) << 8 | (bnd_port[1] as u16);

        Ok(Self {
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
        let from_bytes = Reply::deserialize(&serialize).unwrap();
        assert_eq!(reply, from_bytes);

        let bytes = [5, 0, 0, 1, 127, 0, 0, 1, 0, 80];
        let reply = Reply::deserialize(&bytes).unwrap();
        assert_eq!(reply, Reply::new(reply_opt::SUCCEEDED, addr_type::IP_V4, &[127, 0, 0, 1], 80));
    }
}
