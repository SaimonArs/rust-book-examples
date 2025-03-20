use client::Config;
use client::ESP;


fn main() {
    let config = Config::build(u32::MAX, 1, "127.0.0.1".to_string(), "7878".to_string());

    let mut esp = ESP::build(config);
    esp.start();
}