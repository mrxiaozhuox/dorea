/// 缓存层示例 - 展示带过期时间的数据缓存
/// dorea.examples.cache
/// 
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
/// 
/// 本示例展示：
/// - 使用 setex 设置带过期时间的数据
/// - 模拟缓存命中/未命中场景
use dorea::{client::DoreaClient, value::DataValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;
    db.select("cache-demo").await?;

    println!("=== Dorea 缓存层示例 ===\n");

    // 设置一个 5 秒后过期的缓存
    println!("📦 设置缓存 'user:profile' (5秒过期)...");
    db.setex(
        "user:profile",
        DataValue::String(r#"{"name":"Alice","age":25}"#.to_string()),
        5, // 5 秒后过期
    ).await?;

    // 立即获取 - 应该能拿到
    println!("🔍 立即获取: {:?}", db.get("user:profile").await);

    // 等待 3 秒后获取 - 还能拿到
    println!("⏳ 等待 3 秒...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    println!("🔍 3秒后获取: {:?}", db.get("user:profile").await);

    // 再等待 3 秒 - 应该过期了
    println!("⏳ 再等待 3 秒...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    println!("🔍 6秒后获取: {:?}", db.get("user:profile").await);

    println!("\n✅ 缓存演示完成！");

    // 清理测试数据
    db.clean().await?;
    Ok(())
}
