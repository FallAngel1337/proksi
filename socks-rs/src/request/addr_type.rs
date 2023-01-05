//! # Address Type
//! The possible addresses to fill in the request's `atyp` field

#[allow(missing_docs)]
#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub enum AddrType {
    IpV4 = 0x1,
    DomainName = 0x3,
    IpV6 = 0x4,
}