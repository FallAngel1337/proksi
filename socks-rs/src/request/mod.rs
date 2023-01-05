//! # Requests
//! Contains the `Request` struct according and 
//! to [`RFC 1928`](https://datatracker.ietf.org/doc/html/rfc1928)

pub mod command;
pub mod addr_type;

use std::net::SocketAddr;

use command::Command;
use addr_type::AddrType;
use crate::utils::Sendible;
use crate::SOCKS_VERSION;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Request {
    version: u8,
    cmd: Command,
    rsv: u8, // reserved, always 0x0
    atyp: AddrType,
    dst_addr: Vec<u8>,
    dst_port: u16
}

impl Request {
    pub fn new(cmd: Command, atyp: AddrType, dst_addr: &[u8], dst_port: u16) -> Self {
        Self {
            version: SOCKS_VERSION,
            cmd,
            rsv: 0x0,
            atyp,
            dst_addr: dst_addr.to_vec(),
            dst_port
        }
    }

    pub fn command(&self) -> &Command {
        &self.cmd
    }

    pub fn addr_type(&self) -> &AddrType {
        &self.atyp
    }

    pub fn socket_addr(&self) -> ( &[u8], u16 ) {
        (&self.dst_addr, self.dst_port)
    }
}

impl<'s> Sendible<'s> for Request {}