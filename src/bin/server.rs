use dorea::server::{DOREA_VERSION, Listener};

#[tokio::main]
pub async fn main() {

    const HOSTNAME: &str = "127.0.0.1";
    const PORT: u16 = 3450;

    println!("\n♘「 Dorea database {} 」♖", DOREA_VERSION);
    println!("server address: {}:{} ( tcp service )", HOSTNAME, PORT);
    println!("https://github.com/mrxiaozhuox/Dorea\n");

    let mut listener = Listener::new(HOSTNAME, PORT).await;
    listener.start().await;
}