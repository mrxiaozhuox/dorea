/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
use dorea::{client::DoreaClient, value::DataValue};
use doson::binary::Binary;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;

    // 将 README文件 加载到 Binary 对象中，并准备储存至数据库
    let readme = Binary::from_file(PathBuf::from("./README.md"))?;

    db.setex("foo", DataValue::Binary(readme), 0).await?;

    println!("foo: {:?}.", db.get("foo").await.unwrap_or(DataValue::None));

    Ok(())
}
