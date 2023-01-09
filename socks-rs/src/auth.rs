//! # Auth
//! This moodule contains the struct that describes the
//! user and password authentication.
//! 
//! ## **SECURITY**:
//! 
//! A pointed by the RFC:
//!  "Since the request carries the password in cleartext, this subnegotiation 
//! is not recommended for environments where "sniffing" is possible and practical."

use crate::Sendible;

/// The auth request according to [`RFC 1929`](https://www.rfc-editor.org/rfc/rfc1929)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AuthRequest<'a> {
    /// current version of the subnegotiation (0x1)
    pub version: u8,

    /// length of the `uname` field
    pub ulen: u8,

    /// username
    pub uname: &'a [u8],

    /// length of the `passwd` field
    pub plen: u8,

    /// password
    pub passwd: &'a [u8]
}

/// The auth reponse according to [`RFC 1929`](https://www.rfc-editor.org/rfc/rfc1929)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AuthResponse {
    /// current version of the subnegotiation (0x1)
    pub version: u8,

    /// reponse status
    pub status: u8
}

impl<'a> AuthRequest<'a> {
    /// Created a new auth request
    pub fn new(uname: &'a str, passwd: &'a str) -> Self {
        let (uname, passwd) = (uname.as_bytes(), passwd.as_bytes());

        Self {
            version: 0x1,
            ulen: uname.len() as u8,
            uname,
            plen: passwd.len() as u8,
            passwd
        }
    }
}

impl AuthResponse {
    /// Created a new auth response
    pub fn new(status: u8) -> Self {
        Self {
            version: 0x1,
            status
        }
    }
}


impl<'s> Sendible<'s> for AuthRequest<'s> {
    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        let mut vec = vec![self.version, self.ulen];
        vec.extend(self.uname);
        vec.push(self.plen);
        vec.extend(self.passwd);
        Ok(vec)
    }

    fn deserialize(data: &'s [u8]) -> std::io::Result<Self> {
        let (version, ulen) = (data[0], data[1]);
        
        let offset = ulen as usize + 2;
        
        let uname = &data[2..offset];
        let plen = data[offset];
        let passwd = &data[offset+1..];

        Ok(Self {
            version,
            ulen,
            uname,
            plen,
            passwd
        })
    }
}

impl<'s> Sendible<'s> for AuthResponse {
    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        Ok(vec![self.version, self.status])
    }

    fn deserialize(data: &'s [u8]) -> std::io::Result<Self> {
        let (version, status) = (data[0], data[1]);

        Ok(Self {
            version,
            status
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn auth_request_serr_deser() {
        let auth_request = AuthRequest::new("random_username", "random_password");
        let serialized = auth_request.serialize().unwrap();
        let from_bytes = AuthRequest::deserialize(&serialized).unwrap();
        assert_eq!(auth_request, from_bytes);

        let bytes = [1, 6, 98, 97, 116, 97, 116, 97, 6, 98, 97, 116, 97, 116, 97];
        let auth_request = AuthRequest::deserialize(&bytes).unwrap();
        assert_eq!(auth_request, AuthRequest::new("batata", "batata"));
    }

    #[test]
    fn auth_response_serr_deser() {
        let auth_reponse = AuthResponse::new(0x0);
        let serialized = auth_reponse.serialize().unwrap();
        let from_bytes = AuthResponse::deserialize(&serialized).unwrap();
        assert_eq!(auth_reponse, from_bytes);

        let bytes = [1, 0];
        let auth_reponse = AuthResponse::deserialize(&bytes).unwrap();
        assert_eq!(auth_reponse, AuthResponse::new(0x0));
    }
}
