# Proksi

This project intends to be a full SOCKS v5 based on [`RFC 1928`](https://www.rfc-editor.org/rfc/rfc1928) compliant server library in pure rust.

## Features

- [x] Support for `CONNECT` command
- [x] Support for `BIND` command
- [x] Support for no authentication mode
- [x] Support for user/password authentication

### What is missing at the moment?

- [ ] `UDP ASSOCIATE` command
- [ ] [`GSSAPI`](https://www.rfc-editor.org/rfc/rfc1961.html) authentication method

### Example
```rs
#[tokio::main]
async fn main() {
    let server = Server::default(); // Will be listening at 0.0.0.0:1080
    server.start().await.unwrap();
}
```

You can test the proxy by running `curl`
```
$ curl --socks5 localhost google.com
```