use proksi::{user::User, Server};
use socks_rs::establish::method;
use tokio::time::{self, Duration};

#[tokio::test]
async fn server_curl_test() {
    use tokio::process::Command;

    let server = Server::new(
        "127.0.0.1:1082",
        vec![method::NO_AUTHENTICATION_REQUIRED],
        vec![],
    )
    .unwrap();

    let handler = tokio::spawn(async move { server.start().await.unwrap() });

    time::sleep(Duration::from_secs(2)).await;

    let cmd = Command::new("/bin/curl")
        .args(["--socks5", "localhost:1082", "google.com"])
        .output()
        .await
        .unwrap();

    assert!(cmd.status.success());

    handler.abort();
    time::sleep(Duration::from_secs(1)).await;
    assert!(handler.is_finished());
}

#[tokio::test]
async fn server_curl_userpass_test() {
    use tokio::process::Command;

    let (username, password) = ("admin", "admin");
    let user = User::new(username, password);

    let server = Server::new(
        "127.0.0.1:1083",
        vec![method::USERNAME_PASSWORD],
        vec![user],
    )
    .unwrap();

    let handler = tokio::spawn(async move { server.start().await.unwrap() });
    time::sleep(Duration::from_secs(2)).await;

    let cmd = Command::new("/bin/curl")
        .args([
            "--proxy",
            "socks5://admin:admin@localhost:1083",
            "google.com",
        ])
        .output()
        .await
        .unwrap();

    assert!(cmd.status.success());

    handler.abort();
    time::sleep(Duration::from_secs(1)).await;
    assert!(handler.is_finished());
}
