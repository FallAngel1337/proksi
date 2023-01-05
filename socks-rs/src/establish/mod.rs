//! # Establish
//! This moodule contains the struct that describes the connection establish request
//! that need to be sent to the SOCKS server.

pub(crate) mod methods;
use methods::Methods;
use crate::SOCKS_VERSION;

/// The request packet
pub struct EstablishRequest<'a> {
    version: u8,
    nmethods: u8,
    methods: &'a [Methods]
}

impl<'a> EstablishRequest<'a> {
    /// Constructs a new connection establish request
    pub fn new(methods: &'a [Methods]) -> Self {
        Self {
            version: SOCKS_VERSION,
            nmethods: methods.len() as u8,
            methods
        }
    }
}