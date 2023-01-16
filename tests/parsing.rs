use proksi::Server;

const CONFIG_FILE: &str = r#"
{
    "addr": "127.0.0.1:1080",
    "auth": [0, 2],
    "allowed_users": [
        {
            "username": "alice",
            "password": "1q2w3e4r"
        },
        {
            "username": "bob",
            "password": "p@sSw0rd"
        }
    ]
}"#;

#[test]
fn parsing_test() {
    let server = serde_json::from_str::<Server>(CONFIG_FILE)
        .unwrap();
    println!("{server:#?}");
}