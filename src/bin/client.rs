use dorea::client::{Client, ClientOption};

#[tokio::main]
pub async fn main() {
    let mut client = Client::new("127.0.0.1",3450,ClientOption {
        password: "123456"
    }).await;

    let val = client.get("foo").await;
    println!("{:?}",val);
}