//! # Establish
//! This moodule contains the struct that describes the connection establish request
//! that need to be sent to the SOCKS server.

pub(crate) mod methods;
use methods::Methods;
use crate::SOCKS_VERSION;

/// The request packet
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct EstablishRequest {
    version: u8,
    nmethods: u8,
    methods: Vec<Methods>
}

impl EstablishRequest {
    /// Constructs a new connection establish request
    pub fn new(methods: &[Methods]) -> Self {
        Self {
            version: SOCKS_VERSION,
            nmethods: methods.len() as u8,
            methods: methods.to_vec()
        }
    }

    /// Serialize into a vector of bytes
    pub fn serialize(&self) -> Option<Vec<u8>> {
        bincode::serialize(self)
        .map_or_else(
            |e| { eprintln!("Could not serialize the request! {e:?}"); None },
            Some)
        }
        
    /// Deserialize into a `EstablishRequest`
    pub fn deserialize(data: &[u8]) -> Option<EstablishRequest> {
        bincode::deserialize(data)
        .map_or_else(
            |e| { eprintln!("Could not serialize the request! {e:?}"); None },
            Some)
    }
}

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