//! # Method
//! The methods that can be sent to the SOCKS server

///The values currently defined for METHOD are:
/// 
/// * 0x00 NO AUTHENTICATION REQUIRED (OK)
/// * 0x01 GSSAPI (OK)
/// * 0x02 USERNAME/PASSWORD (OK)
/// * 0x03 to 0x7F IANA ASSIGNED
/// * 0x80 to 0xFE RESERVED FOR PRIVATE METHODS
/// * 0xFF NO ACCEPTABLE METHODS (OK)
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