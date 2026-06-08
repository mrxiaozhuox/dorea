/// 过期时间演示 - 观察数据自动消失
/// dorea.examples.expire-demo
///
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
///
/// 本示例展示：
/// - 不同过期时间的设置
/// - 实时观察数据过期过程
use dorea::{client::DoreaClient, value::DataValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;
    db.select("expire-demo").await?;

    println!("=== Dorea 过期时间演示 ===\n");

    // 设置不同过期时间的数据
    println!("📦 设置三个不同过期时间的数据...\n");

    db.setex("short", DataValue::String("2秒过期".to_string()), 2)
        .await?;
    db.setex("medium", DataValue::String("5秒过期".to_string()), 5)
        .await?;
    db.setex("long", DataValue::String("10秒过期".to_string()), 10)
        .await?;
    db.setex("permanent", DataValue::String("永不过期".to_string()), 0)
        .await?;

    println!("   short   -> 2秒");
    println!("   medium  -> 5秒");
    println!("   long    -> 10秒");
    println!("   permanent -> 永不过期\n");

    // 开始监控
    println!("⏱️  开始监控数据状态...\n");
    println!("时间(s) | short | medium | long | permanent");
    println!("--------|-------|--------|------|----------");

    for t in 0..=12 {
        let short = check_exists(&mut db, "short").await;
        let medium = check_exists(&mut db, "medium").await;
        let long = check_exists(&mut db, "long").await;
        let permanent = check_exists(&mut db, "permanent").await;

        println!(
            "   {:2}   |   {}   |   {}    |  {}  |    {}",
            t,
            status_icon(short),
            status_icon(medium),
            status_icon(long),
            status_icon(permanent)
        );

        if t < 12 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    println!("\n✅ 过期时间演示完成！");
    db.clean().await?;
    Ok(())
}

async fn check_exists(db: &mut DoreaClient, key: &str) -> bool {
    // 同步检查 - 这里用 blocking 方式简化演示
    // 实际应用中应该用 async
    (db.get(key).await).is_some()
}

fn status_icon(exists: bool) -> &'static str {
    if exists {
        "✅"
    } else {
        "❌"
    }
}
