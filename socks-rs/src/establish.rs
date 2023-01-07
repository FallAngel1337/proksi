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


/// The REQUEST packet to establish the connection
#[derive(Debug, Clone)]
pub struct EstablishRequest {
    version: u8,
    nmethods: u8,
    methods: Vec<u8>
}

/// The RESPONSE packet to establish the connection
#[derive(Debug, Clone)]
pub struct EstablishResponse {
    version: u8,
    method: u8
}

impl EstablishRequest {
    /// Constructs a new connection establish REQUEST
    pub fn new(methods: &[u8]) -> Self {
        Self {
            version: SOCKS_VERSION,
            nmethods: methods.len() as u8,
            methods: methods.to_vec()
        }
    }

    /// `methods` field getter
    pub fn methods(&self) -> &[u8] {
        &self.methods
    }
}

impl EstablishResponse {
    /// Constructs a new connection establish RESPONSE
    pub fn new(method: u8) -> Self {
        Self {
            version: SOCKS_VERSION,
            method
        }
    }

    /// `method` field getter
    pub fn method(&self) -> &u8 {
        &self.method
    }
}

impl<'s> Sendible<'s> for EstablishRequest {
    fn serialize(&self) -> Option<Vec<u8>> {
        let mut data = vec![self.version, self.nmethods];
        data.extend(self.methods.iter().cloned());
        Some(data)
    }

    fn deserialize(data: &'s [u8]) -> Option<Self> {
        let (version, nmethods, methods) = (data[0], data[1], (&data[1..]).iter().cloned().collect());

        Some(
            Self {
                version,
                nmethods,
                methods
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn establish_serr_deser() {
        let estbl = EstablishRequest::new(&[Method::NoAuthenticationRequired]);
        let serialized = estbl.serialize().unwrap();
        let new = EstablishRequest::deserialize(&serialized).unwrap();

        assert_eq!(estbl.version, new.version);
        assert_eq!(estbl.nmethods, new.nmethods);
        assert!(estbl.methods.iter().all(|elem| new.methods.contains(elem)));
        assert_eq!(serialized, [5, 1, 1, 0])
    }
}