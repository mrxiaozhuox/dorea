/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！

use dorea::{client::DoreaClient, value::DataValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;

    db.select("demo").await?;

    // try to insert a new key-value: { foo: String("bar") }
    db.setex("foo", DataValue::String(String::from("Hello DoreaDB")), 0).await?;

    println!("foo: {:?}.", db.get("foo").await.unwrap_or(DataValue::None));

    Ok(())
}