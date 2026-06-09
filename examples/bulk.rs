/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
/// dorea.examples.bulk
/// 对于数据进行批量上传：
/// 本 Demo 将对 数据库循环插入 1024 次。
use dorea::{client::DoreaClient, value::DataValue};
use std::time::Instant;

const TOTAL_RECORDS: usize = 20000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;
    db.select("bulk-test").await?;

    println!("=== 单客户端批量插入测试 ===");
    println!("📝 插入数量: {} 条\n", TOTAL_RECORDS);

    let start_time = Instant::now();
    let mut success = 0u64;

    for i in 1..=TOTAL_RECORDS {
        match db
            .setex(&i.to_string(), DataValue::Number(i as f64), 0)
            .await
        {
            Ok(_) => {
                success += 1;
            }
            Err(err) => {
                eprintln!("FAIL at {}: {}", i, err);
            }
        };
    }

    let elapsed = start_time.elapsed();
    
    println!("\n=== 测试结果 ===");
    println!("✅ 成功: {} 条", success);
    println!("⏱️  耗时: {:?}", elapsed);
    println!("🚀 吞吐量: {:.2} ops/s", success as f64 / elapsed.as_secs_f64());

    db.clean().await?;
    Ok(())
}
