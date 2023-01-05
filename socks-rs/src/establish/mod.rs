//! # Establish
//! This moodule contains the struct that describes the connection establish request
//! that need to be sent to the SOCKS server.

pub(crate) mod method;
pub use method::Method;
use crate::{SOCKS_VERSION, utils::*};

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

impl<'s> Sendible<'s> for EstablishRequest {}
impl<'s> Sendible<'s> for EstablishResponse {}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serr_deser() {
        let estbl = EstablishRequest::new(&[Method::NoAuthenticationRequired, Method::UsernamePassword]);
        println!("{estbl:?}");
        
        let new = EstablishRequest::deserialize(&estbl.serialize().unwrap()).unwrap();
        println!("{new:?}");

        assert_eq!(estbl.version, new.version);
        assert_eq!(estbl.nmethods, new.nmethods);
        assert!(estbl.methods.iter().all(|elem| new.methods.contains(elem)));
    }
}