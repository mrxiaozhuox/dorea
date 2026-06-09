/// 消息队列示例 - 简单任务队列
/// dorea.examples.queue
///
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
///
/// 本示例展示：
/// - 使用 List 类型存储队列
/// - EDIT push/remove 实现 FIFO 队列
use dorea::{client::DoreaClient, value::DataValue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;
    db.select("queue-demo").await?;

    println!("=== Dorea 消息队列示例 ===\n");

    // 初始化队列
    println!("📬 初始化任务队列 'task_queue'...");
    let initial_queue: Vec<DataValue> = vec![];
    db.setex("task_queue", DataValue::List(initial_queue), 0)
        .await?;

    // 生产者：添加任务到队列（使用 EDIT push 在末尾添加）
    println!("\n📤 生产者: 添加任务到队列...");
    let tasks = [
        r#"{"task":"send_email","to":"user@example.com"}"#,
        r#"{"task":"generate_report","type":"monthly"}"#,
        r#"{"task":"cleanup_logs","older_than":"7d"}"#,
    ];

    for (i, task) in tasks.iter().enumerate() {
        db.execute(&format!("edit @task_queue push {}", task))
            .await?;
        println!("   任务 #{}: {}", i + 1, task);
    }

    // 查看队列状态
    println!("\n📋 当前队列: {:?}", db.get("task_queue").await);

    // 消费者：从队列取出任务（使用 EDIT remove 0 删除第一个元素，实现 FIFO）
    println!("\n📥 消费者: 处理任务...");
    for i in 1..=3 {
        // 先获取第一个元素
        if let Some(DataValue::List(list)) = db.get("task_queue").await {
            if let Some(task) = list.first() {
                println!("   处理任务 #{}: {:?}", i, task);
            }
        }
        // 再删除第一个元素
        db.execute("edit @task_queue remove 0").await?;
    }

    // 队列应该空了
    println!("\n📭 队列状态: {:?}", db.get("task_queue").await);

    println!("\n✅ 消息队列演示完成！");
    db.clean().await?;
    Ok(())
}
