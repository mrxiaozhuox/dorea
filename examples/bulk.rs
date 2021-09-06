/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！

use dorea::{client::DoreaClient, value::DataValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;

    // db.select("demo").await?;

    for i in 0..200000  {
        println!("NOW: {:?}", i);
        db.setex("TESTVAL", DataValue::Number(i as f64), 0).await.unwrap();
    }


    Ok(())
}
