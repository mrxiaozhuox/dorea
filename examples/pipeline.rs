/// Pipeline 批量插入测试 - 对比普通写入和 Pipeline 写入
/// dorea.examples.pipeline
///
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
///
/// 本示例展示：
/// - 普通逐条写入的性能
/// - Pipeline 批量写入的性能
/// - 对比两种方式的吞吐量
use dorea::client::DoreaClient;
use std::time::Instant;

const TOTAL_RECORDS: usize = 20000;
const BATCH_SIZE: usize = 100; // 每个 pipeline 批次的命令数

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;

    println!("=== Pipeline 批量插入测试 ===\n");
    println!("📝 总数据量: {} 条", TOTAL_RECORDS);
    println!("📦 Pipeline 批次大小: {} 条\n", BATCH_SIZE);

    // ===== 测试 1: 普通逐条写入 =====
    db.select("pipeline-test-normal").await?;
    println!("📊 测试 1: 普通逐条写入...");

    let start_time = Instant::now();
    let mut success = 0u64;

    for i in 1..=TOTAL_RECORDS {
        let cmd = format!("set key_{} \"value_{}\"", i, i);
        match db.execute(&cmd).await {
            Ok((state, _)) if state == dorea::network::NetPacketState::OK => {
                success += 1;
            }
            _ => {}
        }
    }

    let normal_elapsed = start_time.elapsed();
    let normal_ops = success as f64 / normal_elapsed.as_secs_f64();

    println!("   ✅ 成功: {} 条", success);
    println!("   ⏱️  耗时: {:?}", normal_elapsed);
    println!("   🚀 吞吐量: {:.2} ops/s\n", normal_ops);

    db.clean().await?;

    // ===== 测试 2: Pipeline 批量写入 =====
    db.select("pipeline-test-batch").await?;
    println!("📊 测试 2: Pipeline 批量写入...");

    let start_time = Instant::now();
    let mut success = 0u64;

    // 分批处理
    for batch_start in (1..=TOTAL_RECORDS).step_by(BATCH_SIZE) {
        let batch_end = (batch_start + BATCH_SIZE - 1).min(TOTAL_RECORDS);
        
        // 构建这一批的命令
        let commands: Vec<String> = (batch_start..=batch_end)
            .map(|i| format!("set key_{} \"value_{}\"", i, i))
            .collect();
        let cmd_refs: Vec<&str> = commands.iter().map(|s| s.as_str()).collect();

        // Pipeline 发送
        match db.pipeline(&cmd_refs).await {
            Ok(results) => {
                for (state, _) in results {
                    if state == dorea::network::NetPacketState::OK {
                        success += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("Pipeline error: {}", e);
            }
        }
    }

    let pipeline_elapsed = start_time.elapsed();
    let pipeline_ops = success as f64 / pipeline_elapsed.as_secs_f64();

    println!("   ✅ 成功: {} 条", success);
    println!("   ⏱️  耗时: {:?}", pipeline_elapsed);
    println!("   🚀 吞吐量: {:.2} ops/s\n", pipeline_ops);

    db.clean().await?;

    // ===== 对比结果 =====
    println!("=== 对比结果 ===");
    println!("普通写入:    {:>10.2} ops/s", normal_ops);
    println!("Pipeline:    {:>10.2} ops/s", pipeline_ops);
    println!("提升倍数:    {:>10.2}x", pipeline_ops / normal_ops);

    Ok(())
}
