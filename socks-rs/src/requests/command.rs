//! # Command
//! The possible commands to fill in the request's `cmd` field

#[allow(missing_docs)]
#[derive(
    serde::Serialize, serde::Deserialize,
    Debug, Clone, Copy, PartialEq
)]
pub enum Command {
    Connect = 0x1,
    Bind = 0x2,
    UdpAssociate = 0x3,
}