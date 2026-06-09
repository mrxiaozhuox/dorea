/// 并发批量插入示例 - 测试多客户端并发写入
/// dorea.examples.bulk-concurrent
///
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
///
/// 本示例展示：
/// - 多个客户端并发连接
/// - 并发写入数据
/// - 统计总耗时
use dorea::{client::DoreaClient, value::DataValue};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// 并发客户端数量
const CONCURRENT_CLIENTS: usize = 4;

/// 每个客户端插入的数据量（注意：单数据库默认上限 25600）
const RECORDS_PER_CLIENT: usize = 10000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Dorea 并发批量插入测试 ===\n");
    println!("🧵 并发客户端数: {}", CONCURRENT_CLIENTS);
    println!("📝 每客户端插入: {} 条", RECORDS_PER_CLIENT);
    println!("📊 总数据量: {} 条\n", CONCURRENT_CLIENTS * RECORDS_PER_CLIENT);

    let total_success = Arc::new(AtomicU64::new(0));
    let total_failed = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();

    // 创建多个并发任务
    let mut handles = Vec::new();

    for client_id in 0..CONCURRENT_CLIENTS {
        let success = Arc::clone(&total_success);
        let failed = Arc::clone(&total_failed);

        let handle = tokio::spawn(async move {
            // 每个客户端独立连接
            let mut db = match DoreaClient::connect(("127.0.0.1", 3450), "").await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("客户端 {} 连接失败: {}", client_id, e);
                    return;
                }
            };

            // 使用独立的数据库避免单库索引限制
            let db_name = format!("bulk-client-{}", client_id);
            if let Err(e) = db.select(&db_name).await {
                eprintln!("客户端 {} 选择数据库失败: {}", client_id, e);
                return;
            }

            println!("✅ 客户端 {} 已连接，数据库: {}", client_id, db_name);

            // 批量插入
            for i in 0..RECORDS_PER_CLIENT {
                let key = format!("key-{}-{}", client_id, i);
                match db.setex(&key, DataValue::Number(i as f64), 0).await {
                    Ok(_) => {
                        success.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        failed.fetch_add(1, Ordering::Relaxed);
                        if i == 0 || i % 1000 == 0 {
                            eprintln!("❌ 客户端 {} 插入失败 [{}]: {}", client_id, i, e);
                        }
                    }
                }
            }

            // 清理测试数据
            let _ = db.clean().await;
            println!("🏁 客户端 {} 完成", client_id);
        });

        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start_time.elapsed();
    let success = total_success.load(Ordering::Relaxed);
    let failed = total_failed.load(Ordering::Relaxed);

    println!("\n=== 测试结果 ===");
    println!("✅ 成功: {} 条", success);
    println!("❌ 失败: {} 条", failed);
    println!("⏱️  耗时: {:?}", elapsed);
    println!("🚀 吞吐量: {:.2} ops/s", success as f64 / elapsed.as_secs_f64());

    Ok(())
}
