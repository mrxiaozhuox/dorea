/// 计数器示例 - 文章浏览量/点赞统计
/// dorea.examples.counter
/// 
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
/// 
/// 本示例展示：
/// - 使用 Number 类型存储计数
/// - 使用 INCR 命令原子递增
use dorea::{client::DoreaClient, value::DataValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;
    db.select("counter-demo").await?;

    println!("=== Dorea 计数器示例 ===\n");

    // 初始化文章浏览量
    println!("📝 初始化文章浏览量...");
    db.setex("article:views:hello-world", DataValue::Number(0.0), 0).await?;

    // 模拟多次访问
    println!("👥 模拟用户访问...\n");
    for i in 1..=5 {
        // 使用 execute 执行 INCR 命令
        db.execute(&format!("incr article:views:hello-world 1")).await?;
        
        let views = match db.get("article:views:hello-world").await {
            Some(DataValue::Number(n)) => n as i32,
            _ => 0,
        };
        println!("   访问 #{}: 当前浏览量 = {}", i, views);
    }

    // 显示最终结果
    let final_views = match db.get("article:views:hello-world").await {
        Some(DataValue::Number(n)) => n as i32,
        _ => 0,
    };
    println!("\n📊 最终浏览量: {}", final_views);

    // 多个计数器示例
    println!("\n📝 多计数器演示...");
    db.setex("stats:daily_visits", DataValue::Number(100.0), 0).await?;
    db.setex("stats:api_calls", DataValue::Number(50.0), 0).await?;

    db.execute("incr stats:daily_visits 25").await?;
    db.execute("incr stats:api_calls 100").await?;

    println!("   日访问量: {:?}", db.get("stats:daily_visits").await);
    println!("   API 调用: {:?}", db.get("stats:api_calls").await);

    println!("\n✅ 计数器演示完成！");
    db.clean().await?;
    Ok(())
}
