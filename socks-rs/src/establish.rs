//! # Establish
//! This moodule contains the struct that describes the connection establish request
//! that need to be sent to the SOCKS server.

use crate::utils::Sendible;
use crate::SOCKS_VERSION;

#[allow(missing_docs)]
pub mod method {
    pub const NO_AUTHENTICATION_REQUIRED: u8 = 0x0;
    pub const GSSAPI: u8 = 0x1;
    pub const USERNAME_PASSWORD: u8 = 0x2;
    pub const NO_ACCEPTABLE_METHODS: u8 = 0xff;
}

/// The request to establish the connection (client-only)
#[derive(Debug, Clone)]
pub struct EstablishRequest<'a> {
    /// protocol version (0x5)
    pub version: u8,

    /// number of method identifier octets that appear in the METHODS field.
    pub nmethods: u8,

    /// supported methods
    pub methods: &'a [u8],
}

/// The RESPONSE packet to establish the connection (server-only)
#[derive(Debug, Clone)]
pub struct EstablishResponse {
    /// protocol version (0x5)
    pub version: u8,

    /// selected method by server
    pub method: u8,
}

impl<'a> EstablishRequest<'a> {
    /// Constructs a new connection establish REQUEST
    pub fn new(methods: &'a [u8]) -> Self {
        Self {
            version: SOCKS_VERSION,
            nmethods: methods.len() as u8,
            methods,
        }
    }
}

impl EstablishResponse {
    /// Constructs a new connection establish RESPONSE
    pub fn new(method: u8) -> Self {
        Self {
            version: SOCKS_VERSION,
            method,
        }
    }
}

impl<'s> Sendible<'s> for EstablishRequest<'s> {
    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        let mut data = vec![self.version, self.nmethods];
        data.extend(self.methods.iter().cloned());
        Ok(data)
    }

    fn deserialize(data: &'s [u8]) -> std::io::Result<Self> {
        let (version, nmethods, methods) = (data[0], data[1], &data[1..]);

        Ok(Self {
            version,
            nmethods,
            methods,
        })
    }
}

impl<'s> Sendible<'s> for EstablishResponse {
    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        Ok(vec![self.version, self.method])
    }

    fn deserialize(data: &[u8]) -> std::io::Result<Self> {
        let (version, method) = (data[0], data[1]);
        Ok(Self { version, method })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn establish_serr_deser() {
        let estbl = EstablishRequest::new(&[method::NO_AUTHENTICATION_REQUIRED]);
        let serialized = estbl.serialize().unwrap();
        let new = EstablishRequest::deserialize(&serialized).unwrap();

        assert_eq!(estbl.version, new.version);
        assert_eq!(estbl.nmethods, new.nmethods);
        assert!(estbl.methods.iter().all(|elem| new.methods.contains(elem)));
        assert_eq!(serialized, [5, 1, 0])
    }
}
