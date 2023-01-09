//! # Requests
//! Contains the `Request` struct according and
//! to [`RFC 1928`](https://datatracker.ietf.org/doc/html/rfc1928)

use crate::utils::Sendible;
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

/// 
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Request<'a> {
    /// protocol version (0x5)
    pub version: u8,

    /// command
    pub cmd: u8,

    /// RESERVED
    pub rsv: u8,

    /// address type
    pub atyp: u8,

    ///  desired destination address
    pub dst_addr: &'a [u8],

    /// desired destination port in network octet order
    pub dst_port: u16,
}

#[allow(unused)]
impl<'a> Request<'a> {
    /// Creates a new request (client-only)
    pub fn new(cmd: u8, atyp: u8, dst_addr: &'a [u8], dst_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            cmd,
            rsv: 0x0,
            atyp,
            dst_addr,
            dst_port,
        }
    }
}

impl<'s> Sendible<'s> for Request<'s> {
    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        let mut data = vec![self.version, self.cmd, self.rsv, self.atyp];
        data.extend(self.dst_addr);
        data.extend([
            ((self.dst_port >> 8) & 0xff) as u8,
            (self.dst_port & 0xff) as u8,
        ]);
        Ok(data)
    }

    fn deserialize(data: &'s [u8]) -> std::io::Result<Self> {
        let (version, cmd, rsv, atyp) = (data[0], data[1], data[2], data[3]);

        let offset = match atyp {
            addr_type::IP_V4 => 8_usize,
            addr_type::DOMAIN_NAME => panic!("Can't do DOMAINNAME yet"),
            addr_type::IP_V6 => 20_usize,
            _ => panic!("Invalid address type"),
        };

        let dst_addr = &data[4..offset];
        let dst_port = &data[offset..];
        let dst_port = (dst_port[0] as u16) << 8 | (dst_port[1] as u16);

        Ok(Self {
            version,
            cmd,
            rsv,
            atyp,
            dst_addr,
            dst_port,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn request_serr_deser() {
        let request = Request::new(command::CONNECT, addr_type::IP_V4, &[127, 0, 0, 1], 1080);
        let serialized = request.serialize().unwrap();
        let new = Request::deserialize(&serialized).unwrap();

        println!("{serialized:?}");
        println!("{request:?}\n{new:?}");

        assert_eq!(request, new);
    }
}
