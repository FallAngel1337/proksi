//! # Establish
//! This moodule contains the struct that describes the connection establish request
//! that need to be sent to the SOCKS server.

pub(crate) mod methods;
use methods::Methods;
use crate::{SOCKS_VERSION, Sendible};

/// The REQUEST packet to establish the connection
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct EstablishRequest {
    version: u8,
    nmethods: u8,
    methods: Vec<Methods>
}

/// The RESPONSE packet to establish the connection
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct EstablishResponse {
    version: u8,
    method: Methods
}

impl EstablishRequest {
    /// Constructs a new connection establish REQUEST
    pub fn new(methods: &[Methods]) -> Self {
        Self {
            version: SOCKS_VERSION,
            nmethods: methods.len() as u8,
            methods: methods.to_vec()
        }
    }
}

impl EstablishResponse {
    /// Constructs a new connection establish RESPONSE
    pub fn new(method: Methods) -> Self {
        Self {
            version: SOCKS_VERSION,
            method
        }
    }
}

impl<'s> Sendible<'s> for EstablishRequest {}
impl<'s> Sendible<'s> for EstablishResponse {}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serr_deser() {
        let estbl = EstablishRequest::new(&[Methods::NoAuthenticationRequired, Methods::UsernamePassword]);
        println!("{estbl:?}");
        
        let new = EstablishRequest::deserialize(&estbl.serialize().unwrap()).unwrap();
        println!("{new:?}");

        assert_eq!(estbl.version, new.version);
        assert_eq!(estbl.nmethods, new.nmethods);
        assert!(estbl.methods.iter().all(|elem| new.methods.contains(elem)));
    }
}