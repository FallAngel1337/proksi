//! # Establish
//! This moodule contains the struct that describes the connection establish request
//! that need to be sent to the SOCKS server.

use crate::SOCKS_VERSION;

#[allow(missing_docs)]
#[repr(u8)]
#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]

pub enum Method {
    NoAuthenticationRequired = 0x0,
    Gssapi = 0x1,
    UsernamePassword = 0x2,
    NoAcceptableMethods = 0xff,
}


/// The REQUEST packet to establish the connection
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct EstablishRequest {
    version: u8,
    nmethods: u8,
    methods: Vec<Method>
}

/// The RESPONSE packet to establish the connection
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct EstablishResponse {
    version: u8,
    method: Method
}

impl EstablishRequest {
    /// Constructs a new connection establish REQUEST
    pub fn new(methods: &[Method]) -> Self {
        Self {
            version: SOCKS_VERSION,
            nmethods: methods.len() as u8,
            methods: methods.to_vec()
        }
    }

    /// `methods` field getter
    pub fn methods(&self) -> &[Method] {
        &self.methods
    }
}

impl EstablishResponse {
    /// Constructs a new connection establish RESPONSE
    pub fn new(method: Method) -> Self {
        Self {
            version: SOCKS_VERSION,
            method
        }
    }

    /// `method` field getter
    pub fn method(&self) -> &Method {
        &self.method
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::Sendible;

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