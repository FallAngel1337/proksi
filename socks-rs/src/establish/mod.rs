//! # Establish
//! This moodule contains the struct that describes the connection establish request
//! that need to be sent to the SOCKS server.

pub(crate) mod methods;
use methods::Methods;

/// The request packet
pub struct EstablishRequest {
    version: u8,
    nmethods: u8,
    methods: Vec<Methods>
}