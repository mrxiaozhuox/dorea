use dorea::server::{ Listener };

#[tokio::main]
pub async fn main() {
    let mut listener = Listener::new("127.0.0.1", 3450).await;
    listener.start().await;
}