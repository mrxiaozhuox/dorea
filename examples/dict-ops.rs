/// 字典操作示例 - 存储用户配置/Profile
/// dorea.examples.dict-ops
/// 
/// 在你运行这个 Demo 之前，请确保 Dorea 服务已经正常启动！
/// 
/// 本示例展示：
/// - Dict 类型的 CRUD 操作
/// - 嵌套字典结构
use dorea::{client::DoreaClient, value::DataValue};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db = DoreaClient::connect(("127.0.0.1", 3450), "").await?;
    db.select("dict-demo").await?;

    println!("=== Dorea 字典操作示例 ===\n");

    // 创建用户配置
    println!("👤 创建用户配置...\n");
    
    let mut user_settings = HashMap::new();
    
    // 基本信息
    user_settings.insert("name".to_string(), DataValue::String("Alice".to_string()));
    user_settings.insert("email".to_string(), DataValue::String("alice@example.com".to_string()));
    user_settings.insert("age".to_string(), DataValue::Number(25.0));
    user_settings.insert("verified".to_string(), DataValue::Boolean(true));
    
    // 嵌套字典：偏好设置
    let mut preferences = HashMap::new();
    preferences.insert("theme".to_string(), DataValue::String("dark".to_string()));
    preferences.insert("language".to_string(), DataValue::String("zh-CN".to_string()));
    preferences.insert("notifications".to_string(), DataValue::Boolean(true));
    user_settings.insert("preferences".to_string(), DataValue::Dict(preferences));
    
    // 列表：标签
    let tags = vec![
        DataValue::String("rust".to_string()),
        DataValue::String("developer".to_string()),
        DataValue::String("opensource".to_string()),
    ];
    user_settings.insert("tags".to_string(), DataValue::List(tags));

    // 保存
    db.setex("user:alice", DataValue::Dict(user_settings.clone()), 0).await?;
    println!("   ✅ 已保存 user:alice");

    // 读取并展示
    println!("\n📖 读取用户配置...\n");
    match db.get("user:alice").await {
        Some(DataValue::Dict(data)) => {
            println!("   姓名: {:?}", data.get("name"));
            println!("   邮箱: {:?}", data.get("email"));
            println!("   年龄: {:?}", data.get("age"));
            println!("   已验证: {:?}", data.get("verified"));
            println!("   标签: {:?}", data.get("tags"));
            println!("   偏好: {:?}", data.get("preferences"));
        }
        _ => println!("   ❌ 未找到数据"),
    }

    // 更新操作
    println!("\n✏️ 更新用户年龄...");
    let mut updated = user_settings.clone();
    updated.insert("age".to_string(), DataValue::Number(26.0));
    db.setex("user:alice", DataValue::Dict(updated), 0).await?;
    
    match db.get("user:alice").await {
        Some(DataValue::Dict(data)) => {
            println!("   新年龄: {:?}", data.get("age"));
        }
        _ => {}
    }

    // 删除字段
    println!("\n🗑️ 删除用户配置...");
    db.delete("user:alice").await?;
    println!("   ✅ 已删除 user:alice");

    println!("\n✅ 字典操作演示完成！");
    db.clean().await?;
    Ok(())
}
