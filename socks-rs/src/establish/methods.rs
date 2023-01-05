//! # Methods
//! The methods that can be sent to the SOCKS server

///The values currently defined for METHOD are:
/// 
/// * X'00' NO AUTHENTICATION REQUIRED (OK)
/// * X'01' GSSAPI (OK)
/// * X'02' USERNAME/PASSWORD (OK)
/// * X'03' to X'7F' IANA ASSIGNED
/// * X'80' to X'FE' RESERVED FOR PRIVATE METHODS
/// * X'FF' NO ACCEPTABLE METHODS (OK)
#[repr(u8)]
#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub enum Methods {
    NoAuthenticationRequired = 0x0,
    Gssapi = 0x1,
    UsernamePassword = 0x2,
    NoAcceptableMethods = 0xff,
}