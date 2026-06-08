/// 用户会话管理示例 - 模拟登录/登出/检查会话
/// dorea.examples.session
/// 
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
/// 
/// 本示例展示：
/// - 使用 Dict 类型存储复杂会话数据
/// - 模拟用户登录、验证、登出流程
use dorea::{client::DoreaClient, value::DataValue};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;
    db.select("session-demo").await?;

    println!("=== Dorea 会话管理示例 ===\n");

    // 模拟用户登录
    println!("🔐 用户 'alice' 登录...");
    let mut session_data = HashMap::new();
    session_data.insert("user_id".to_string(), DataValue::String("u_12345".to_string()));
    session_data.insert("username".to_string(), DataValue::String("alice".to_string()));
    session_data.insert("role".to_string(), DataValue::String("admin".to_string()));
    session_data.insert("login_time".to_string(), DataValue::Number(1717900000.0));

    let session_id = "sess_abc123";
    db.setex(
        &format!("session:{}", session_id),
        DataValue::Dict(session_data),
        3600, // 1 小时过期
    ).await?;

    // 验证会话
    println!("🔍 验证会话 '{}'...", session_id);
    match db.get(&format!("session:{}", session_id)).await {
        Some(DataValue::Dict(data)) => {
            println!("   ✅ 会话有效！");
            println!("   用户: {:?}", data.get("username"));
            println!("   角色: {:?}", data.get("role"));
        }
        _ => {
            println!("   ❌ 会话无效或已过期");
        }
    }

    // 模拟登出 - 删除会话
    println!("\n🚪 用户登出...");
    db.delete(&format!("session:{}", session_id)).await?;

    // 再次验证
    println!("🔍 再次验证会话...");
    match db.get(&format!("session:{}", session_id)).await {
        Some(_) => println!("   ❌ 会话仍然存在（异常）"),
        None => println!("   ✅ 会话已销毁"),
    }

    println!("\n✅ 会话管理演示完成！");
    db.clean().await?;
    Ok(())
}
