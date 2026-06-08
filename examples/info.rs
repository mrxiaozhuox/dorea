/// 服务器信息示例 - 查看数据库状态
/// dorea.examples.info
/// 
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
/// 
/// 本示例展示：
/// - 使用 info() 方法获取服务器信息
/// - InfoType 枚举的各种选项
use dorea::{client::DoreaClient, InfoType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;
    db.select("info-demo").await?;

    println!("=== Dorea 服务器信息 ===\n");

    // 服务器版本
    let version = db.info(InfoType::ServerVersion).await?;
    println!("📌 服务版本: {}", version.trim());

    // 启动时间
    let startup_time = db.info(InfoType::ServerStartupTime).await?;
    println!("⏰ 启动时间: {}", startup_time.trim());

    // 连接信息
    let current_conn = db.info(InfoType::CurrentConnectionNumber).await?;
    let max_conn = db.info(InfoType::MaxConnectionNumber).await?;
    println!("🔗 当前连接: {} / {}", current_conn.trim(), max_conn.trim());

    // 当前数据库
    let current_db = db.info(InfoType::CurrentDataBase).await?;
    println!("💾 当前数据库: {}", current_db.trim());

    // 预加载数据库列表
    let preload_list = db.info(InfoType::PreloadDatabaseList).await?;
    println!("📂 预加载数据库: {}", preload_list.trim());

    // 索引信息
    let index_info = db.info(InfoType::TotalIndexNumber).await?;
    println!("📊 索引使用: {}", index_info.trim());

    // 当前数据库的 key 列表
    let keys = db.info(InfoType::KeyList).await?;
    println!("🔑 当前 Key 列表: {}", keys.trim());

    println!("\n✅ 信息查询完成！");
    Ok(())
}
