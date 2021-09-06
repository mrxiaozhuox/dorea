/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！

use dorea::{client::DoreaClient, value::DataValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;

    // db.select("demo").await?;

    db.setex("1", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("2", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("3", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("4", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("5", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("6", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("7", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("8", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("8", DataValue::Number(2 as f64), 0).await.unwrap();
    db.setex("8", DataValue::Number(3 as f64), 0).await.unwrap();
    db.setex("9", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("10", DataValue::Number(1 as f64), 0).await.unwrap();
    db.setex("10", DataValue::Number(2 as f64), 0).await.unwrap();

    for i in 0..  {
        println!("NOW: {:?}", i);
        db.setex("4", DataValue::Number(i as f64), 0).await.unwrap();
    }


    Ok(())
}
