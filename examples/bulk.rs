/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
/// dorea.examples.bulk
/// 对于数据进行批量上传：
/// 本 Demo 将对 数据库循环插入 1024 次。

use dorea::{client::DoreaClient, value::DataValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;

    // 循环 1024 次：
    for i in 1..1025  {
        println!("NOW: {:?}", i);
        // 将 {i} Key 设置为一个 Number 数据：{i}
        db.setex(&i.to_string(), DataValue::Number(i as f64), 0).await.unwrap();
    }

    Ok(())
}
