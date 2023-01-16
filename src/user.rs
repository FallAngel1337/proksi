use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[allow(missing_docs, unused)]
impl User {
    #[inline]
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}
