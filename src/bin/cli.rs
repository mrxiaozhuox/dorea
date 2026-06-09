//! Dorea-CLI
//! Author: YuKun Liu<mrxzx.info@gmail.com>
//! Date: 2021/10/25
//! @DoreaDB Client

use clap::{App, Arg, SubCommand};
use colored::Colorize;
use dorea::client::DoreaClient;
use dorea::network::NetPacketState;
use dorea::value::DataValue;
use doson::binary::Binary;
use rustyline::Editor;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

const DOREA_CLI_VERSION: &str = "0.5.0";

/// 打印启动 Banner
fn print_banner() {
    println!();
    println!(
        "{}",
        "  +--------------------------------------------------------+".bright_cyan()
    );
    println!(
        "{}",
        "  |                                                        |".bright_cyan()
    );
    println!(
        "{}",
        "  |    /$$$$$$$                                            |".bright_cyan()
    );
    println!(
        "{}",
        "  |   | $$__  $$                                           |".bright_cyan()
    );
    println!(
        "{}",
        "  |   | $$  \\ $$  /$$$$$$   /$$$$$$   /$$$$$$   /$$$$$$    |".bright_cyan()
    );
    println!(
        "{}",
        "  |   | $$  | $$ /$$__  $$ /$$__  $$ /$$__  $$ |____  $$   |".bright_cyan()
    );
    println!(
        "{}",
        "  |   | $$  | $$| $$  \\ $$| $$  \\__/| $$$$$$$$  /$$$$$$$   |".bright_cyan()
    );
    println!(
        "{}",
        "  |   | $$  | $$| $$  | $$| $$      | $$_____/ /$$__  $$   |".bright_cyan()
    );
    println!(
        "{}",
        "  |   | $$$$$$$/|  $$$$$$/| $$      |  $$$$$$$|  $$$$$$$   |".bright_cyan()
    );
    println!(
        "{}",
        "  |   |_______/  \\______/ |__/       \\_______/ \\_______/   |".bright_cyan()
    );
    println!(
        "{}",
        "  |                                                        |".bright_cyan()
    );
    println!(
        "{}",
        "  |     A Key-Value Storage System                         |".bright_cyan()
    );
    println!(
        "{}",
        "  |                                                        |".bright_cyan()
    );
    println!(
        "{}",
        "  +--------------------------------------------------------+".bright_cyan()
    );
    println!();
    println!("  {} {}", "Version:".dimmed(), DOREA_CLI_VERSION.green());
    println!(
        "  {} {}",
        "Hint:".dimmed(),
        "Type 'docs' to see available commands".yellow()
    );
    println!();
}

/// 智能格式化输出
fn smart_format(data: &str) {
    if data.is_empty() {
        println!("{} {}", "✓".green().bold(), "OK".white());
        return;
    }

    // 尝试解析为 JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
        print_json(&json);
        return;
    }

    // 尝试解析为 Doson（包含 tuple 等非标准 JSON）
    if is_doson_format(data) {
        print_doson(data);
        return;
    }

    // 检测是否是键列表格式（如 info keys 返回的）
    if data.starts_with('[') && data.contains("\",") {
        if let Some(keys) = parse_key_list(data) {
            print_key_table(&keys);
            return;
        }
    }

    // 检测是否是键值对格式（如 info 返回的）
    if data.contains(':') && data.lines().count() > 1 {
        if let Some(pairs) = parse_info_pairs(data) {
            print_info_table(&pairs);
            return;
        }
    }

    // 默认直接输出
    println!("{} {}", "→".bright_cyan(), data.white());
}

/// 检测是否是 doson 格式（包含 tuple 等非标准 JSON）
fn is_doson_format(data: &str) -> bool {
    let trimmed = data.trim();
    trimmed.starts_with('{') && trimmed.contains('(') && trimmed.contains(", ")
}

/// 打印 Doson 格式（带语法高亮）
fn print_doson(data: &str) {
    println!();
    println!(
        "{}",
        "+----------------------------------------+".bright_cyan()
    );
    print_doson_value(data, "");
    println!(
        "{}",
        "+----------------------------------------+".bright_cyan()
    );
}

/// 递归打印 Doson 值
fn print_doson_value(data: &str, indent: &str) {
    let trimmed = data.trim();

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        // Dict
        print_doson_dict(trimmed, indent);
    } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
        // List
        print_doson_list(trimmed, indent);
    } else if trimmed.starts_with('(') && trimmed.ends_with(')') {
        // Tuple
        print_doson_tuple(trimmed, indent);
    } else {
        // 基本类型
        print_doson_primitive(trimmed);
    }
}

/// 打印 Dict
fn print_doson_dict(data: &str, indent: &str) {
    let content = &data[1..data.len() - 1]; // 只去掉最外层的 { }
    let entries = parse_doson_dict_entries(content);

    for (i, (key, value)) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let prefix = if is_last { "+--" } else { "|--" };
        let key_clean = key.trim_matches('"');

        let child_indent = format!("{}    ", indent);

        // 如果是嵌套结构，先打印 key，然后展开 value
        if value.trim().starts_with('{')
            || value.trim().starts_with('[')
            || value.trim().starts_with('(')
        {
            println!(
                "{}{} {}:",
                indent,
                prefix.bright_cyan(),
                key_clean.bright_blue().bold()
            );
            print_doson_value(value, &child_indent);
        } else {
            print!(
                "{}{} {}: ",
                indent,
                prefix.bright_cyan(),
                key_clean.bright_blue().bold()
            );
            print_doson_value(value, "");
            println!();
        }
    }
}

/// 打印 List
fn print_doson_list(data: &str, indent: &str) {
    let content = &data[1..data.len() - 1]; // 只去掉最外层的 [ ]
    let items = parse_doson_list_items(content);

    for (i, item) in items.iter().enumerate() {
        let is_last = i == items.len() - 1;
        let prefix = if is_last { "+--" } else { "|--" };
        let num = format!("[{}]", i);

        let child_indent = format!("{}    ", indent);

        if item.trim().starts_with('{')
            || item.trim().starts_with('[')
            || item.trim().starts_with('(')
        {
            println!("{}{} {}:", indent, prefix.bright_cyan(), num.dimmed());
            print_doson_value(item, &child_indent);
        } else {
            print!("{}{} {}: ", indent, prefix.bright_cyan(), num.dimmed());
            print_doson_value(item, "");
            println!();
        }
    }
}

/// 打印 Tuple
fn print_doson_tuple(data: &str, indent: &str) {
    let content = &data[1..data.len() - 1]; // 只去掉最外层的 ( )
                                            // tuple 只有两个元素，用 ", " 分割
    let parts: Vec<&str> = content.splitn(2, ", ").collect();

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;
        let prefix = if is_last { "+--" } else { "|--" };
        let label = format!(".{}", i);

        let child_indent = format!("{}    ", indent);

        if part.trim().starts_with('{')
            || part.trim().starts_with('[')
            || part.trim().starts_with('(')
        {
            println!("{}{} {}:", indent, prefix.bright_cyan(), label.dimmed());
            print_doson_value(part, &child_indent);
        } else {
            print!("{}{} {}: ", indent, prefix.bright_cyan(), label.dimmed());
            print_doson_value(part, "");
            println!();
        }
    }
}

/// 打印基本类型
fn print_doson_primitive(data: &str) {
    let trimmed = data.trim();

    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        // String
        print!("{}", trimmed.trim_matches('"').green());
    } else if trimmed == "true" || trimmed == "false" {
        // Boolean
        print!("{}", trimmed.yellow());
    } else if trimmed.parse::<f64>().is_ok() {
        // Number
        print!("{}", trimmed.cyan());
    } else {
        print!("{}", trimmed.white());
    }
}

/// 解析 Dict 的键值对
fn parse_doson_dict_entries(content: &str) -> Vec<(String, String)> {
    let mut entries = Vec::new();
    let mut depth = 0;
    let mut current_key = String::new();
    let mut current_value = String::new();
    let mut in_key = false;
    let mut in_value = false;
    let mut in_string = false;

    for ch in content.chars() {
        if ch == '"' && depth == 0 {
            in_string = !in_string;
            if !in_value {
                in_key = true;
            }
        } else if ch == ':' && !in_string && depth == 0 {
            in_key = false;
            in_value = true;
        } else if ch == ',' && !in_string && depth == 0 {
            if !current_key.is_empty() && !current_value.is_empty() {
                entries.push((
                    current_key.trim().to_string(),
                    current_value.trim().to_string(),
                ));
            }
            current_key.clear();
            current_value.clear();
            in_key = false;
            in_value = false;
        } else {
            match ch {
                '{' | '[' | '(' => depth += 1,
                '}' | ']' | ')' => depth -= 1,
                _ => {}
            }
            if in_key {
                current_key.push(ch);
            } else if in_value {
                current_value.push(ch);
            }
        }
    }

    if !current_key.is_empty() && !current_value.is_empty() {
        entries.push((
            current_key.trim().to_string(),
            current_value.trim().to_string(),
        ));
    }

    entries
}

/// 解析 List 的元素
fn parse_doson_list_items(content: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    let mut in_string = false;

    for ch in content.chars() {
        if ch == '"' {
            in_string = !in_string;
            current.push(ch);
        } else if ch == ',' && !in_string && depth == 0 {
            if !current.trim().is_empty() {
                items.push(current.trim().to_string());
            }
            current.clear();
        } else {
            match ch {
                '{' | '[' | '(' => depth += 1,
                '}' | ']' | ')' => depth -= 1,
                _ => {}
            }
            current.push(ch);
        }
    }

    if !current.trim().is_empty() {
        items.push(current.trim().to_string());
    }

    items
}

/// 解析键列表
fn parse_key_list(data: &str) -> Option<Vec<String>> {
    // 尝试解析 JSON 数组
    if let Ok(serde_json::Value::Array(arr)) = serde_json::from_str(data) {
        return Some(
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        );
    }

    // 尝试解析 ["a","b","c"] 格式
    let trimmed = data.trim().trim_start_matches('[').trim_end_matches(']');
    let items: Vec<String> = trimmed
        .split("\",")
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if items.is_empty() {
        return None;
    }
    Some(items)
}

/// 解析信息键值对
fn parse_info_pairs(data: &str) -> Option<Vec<(String, String)>> {
    let pairs: Vec<(String, String)> = data
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect();

    if pairs.is_empty() {
        return None;
    }
    Some(pairs)
}

/// 打印 JSON（带语法高亮）
fn print_json(value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            println!();
            println!(
                "  {}",
                "+----------------------------------------+".bright_cyan()
            );
            for (i, (key, val)) in map.iter().enumerate() {
                let is_last = i == map.len() - 1;
                let prefix = if is_last { "+--" } else { "|--" };
                print_json_kv(key, val, prefix);
            }
            println!(
                "  {}",
                "+----------------------------------------+".bright_cyan()
            );
        }
        serde_json::Value::Array(arr) => {
            println!();
            println!(
                "  {} {} {} items",
                "📋".yellow(),
                "Array:".white(),
                arr.len().to_string().cyan()
            );
            println!("  {}", "----------------------------------------".dimmed());
            for (i, item) in arr.iter().enumerate() {
                let num = format!("[{}]", i).dimmed();
                match item {
                    serde_json::Value::String(s) => println!("  {} {}", num, s.green()),
                    serde_json::Value::Number(n) => println!("  {} {}", num, n.to_string().cyan()),
                    serde_json::Value::Bool(b) => println!("  {} {}", num, b.to_string().yellow()),
                    _ => println!("  {} {}", num, item.to_string().white()),
                }
            }
        }
        _ => {
            println!("{}", highlight_value(value));
        }
    }
}

/// 打印 JSON 键值对（支持递归展开）
fn print_json_kv(key: &str, value: &serde_json::Value, prefix: &str) {
    let key_colored = key.bright_blue().bold();
    match value {
        serde_json::Value::String(s) => {
            println!("  {}: {}: {}", prefix.bright_cyan(), key_colored, s.green());
        }
        serde_json::Value::Number(n) => {
            println!(
                "  {}: {}: {}",
                prefix.bright_cyan(),
                key_colored,
                n.to_string().cyan()
            );
        }
        serde_json::Value::Bool(b) => {
            println!(
                "  {}: {}: {}",
                prefix.bright_cyan(),
                key_colored,
                b.to_string().yellow()
            );
        }
        serde_json::Value::Null => {
            println!(
                "  {}: {}: {}",
                prefix.bright_cyan(),
                key_colored,
                "null".dimmed()
            );
        }
        serde_json::Value::Array(arr) => {
            println!(
                "  {}: {}: {} {} {}",
                prefix.bright_cyan(),
                key_colored,
                "[".white(),
                arr.len().to_string().cyan(),
                "]".white()
            );
            // 递归展开数组
            let child_prefix = format!("{}    ", prefix);
            for (i, item) in arr.iter().enumerate() {
                let item_key = format!("[{}]", i);
                print_json_kv(&item_key, item, &child_prefix);
            }
        }
        serde_json::Value::Object(obj) => {
            println!(
                "  {}: {}: {} {} {}",
                prefix.bright_cyan(),
                key_colored,
                "{".white(),
                obj.len().to_string().cyan(),
                "}".white()
            );
            // 递归展开对象
            let child_prefix = format!("{}    ", prefix);
            for (k, v) in obj {
                print_json_kv(k, v, &child_prefix);
            }
        }
    }
}

/// 高亮值
fn highlight_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.green().to_string(),
        serde_json::Value::Number(n) => n.to_string().cyan().to_string(),
        serde_json::Value::Bool(b) => b.to_string().yellow().to_string(),
        serde_json::Value::Null => "null".dimmed().to_string(),
        _ => value.to_string().white().to_string(),
    }
}

/// 打印键列表表格
fn print_key_table(keys: &[String]) {
    println!();
    println!(
        "  {} {} {}",
        "🔑".yellow(),
        "Keys:".white(),
        keys.len().to_string().cyan()
    );
    println!(
        "  {}",
        "+--------------------------------------------------+".bright_cyan()
    );

    if keys.is_empty() {
        println!(
            "  {} {:^46} {}",
            "|".bright_cyan(),
            "(empty)".dimmed(),
            "|".bright_cyan()
        );
    } else {
        for (i, key) in keys.iter().enumerate() {
            let num = format!("[{:>3}]", i + 1).dimmed();
            let key_display = if key.len() > 40 {
                format!("{:<40}", &format!("{}...", &key[..37]))
            } else {
                format!("{:<40}", key)
            };
            println!(
                "  {} {} {} {}",
                "|".bright_cyan(),
                num,
                key_display.white(),
                "|".bright_cyan()
            );
        }
    }

    println!(
        "  {}",
        "+--------------------------------------------------+".bright_cyan()
    );
}

/// 打印信息表格
fn print_info_table(pairs: &[(String, String)]) {
    println!();
    println!("  {}", "📊 Info".yellow().bold());
    println!(
        "  {}",
        "+------------------+----------------------------------------+".bright_cyan()
    );

    for (key, value) in pairs {
        let key_display = format!("{:<16}", key);
        let value_display = if value.len() > 36 {
            format!("{:<36}", &format!("{}...", &value[..33]))
        } else {
            format!("{:<36}", value)
        };
        println!(
            "  {} {} {} {} {}",
            "|".bright_cyan(),
            key_display.bright_blue(),
            "|".bright_cyan(),
            value_display.white(),
            "|".bright_cyan()
        );
    }

    println!(
        "  {}",
        "+------------------+----------------------------------------+".bright_cyan()
    );
}

/// 打印错误
fn print_err(msg: &str) {
    println!("{} {}", "✗".red().bold(), msg.red());
}

/// 打印 docs 文档
fn print_docs(content: &str) {
    println!();
    println!(
        "{}",
        "┌─────────────────────────────────────────────────────┐".bright_cyan()
    );
    println!(
        "{}",
        "│               📚 Dorea Command Docs                 │".bright_cyan()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────────┘".bright_cyan()
    );
    println!();

    let mut in_code = false;
    for line in content.lines() {
        let t = line.trim();

        if t.starts_with('#') && !t.starts_with("##") {
            println!();
            println!(
                "  {} {}",
                "▸".bright_cyan(),
                t.trim_start_matches('#').trim().white().bold()
            );
            println!("  {}", "─".repeat(50).dimmed());
        } else if t.starts_with("```") {
            in_code = !in_code;
            if in_code {
                println!();
            }
        } else if in_code {
            println!("    {}", t.bright_black().italic());
        } else if !t.is_empty() {
            println!("  {}", t.dimmed());
        }
    }

    println!();
    println!("{}", "─".repeat(55).dimmed());
    println!(
        "  {} Try {} for specific command",
        "💡".yellow(),
        "'docs <command>'".yellow().bold()
    );
    println!();
}

#[tokio::main]
pub async fn main() {
    let matches = App::new("Dorea CLI")
        .version(DOREA_CLI_VERSION)
        .author("YuKun Liu <mrxzx.info@gmail.com>")
        .about("DoreaDB Cli Tool")
        .arg(
            Arg::with_name("HOSTNAME")
                .short("h")
                .long("hostname")
                .takes_value(true)
                .help("Server hostname")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::with_name("PORT")
                .short("p")
                .long("port")
                .takes_value(true)
                .help("Server port")
                .default_value("3450"),
        )
        .arg(
            Arg::with_name("PASSWORD")
                .short("a")
                .long("password")
                .takes_value(true)
                .help("Connection password")
                .default_value(""),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run a single command")
                .arg(Arg::with_name("COMMAND").required(true).index(1))
                .arg(
                    Arg::with_name("HOSTNAME")
                        .short("h")
                        .long("hostname")
                        .takes_value(true)
                        .default_value("127.0.0.1"),
                )
                .arg(
                    Arg::with_name("PORT")
                        .short("p")
                        .long("port")
                        .takes_value(true)
                        .default_value("3450"),
                )
                .arg(
                    Arg::with_name("PASSWORD")
                        .short("a")
                        .long("password")
                        .takes_value(true)
                        .default_value(""),
                )
                .arg(
                    Arg::with_name("DATABASE")
                        .short("t")
                        .long("database")
                        .takes_value(true)
                        .default_value("default"),
                ),
        )
        .get_matches();

    // 单次执行模式
    if let Some(m) = matches.subcommand_matches("run") {
        let hostname = m.value_of("HOSTNAME").unwrap();
        let port = m.value_of("PORT").unwrap();
        let password = m.value_of("PASSWORD").unwrap();
        let target = m.value_of("DATABASE").unwrap();
        let command = m.value_of("COMMAND").unwrap();

        let client = DoreaClient::connect(
            (
                Box::leak(hostname.to_string().into_boxed_str()),
                port.parse::<u16>().unwrap_or(3450),
            ),
            password,
        )
        .await;

        let mut client = match client {
            Ok(c) => c,
            Err(e) => {
                print_err(&format!("Connection failed: {:?}", e));
                exit(1);
            }
        };

        client.select(target).await.ok();
        let res = execute(command, &mut client).await;

        if res.0 == NetPacketState::ERR {
            print_err(&res.1);
        } else {
            smart_format(&res.1);
        }
        return;
    }

    // 交互模式
    print_banner();

    let hostname = matches.value_of("HOSTNAME").unwrap();
    let port = matches.value_of("PORT").unwrap();
    let password = matches.value_of("PASSWORD").unwrap();

    let client = DoreaClient::connect(
        (
            Box::leak(hostname.to_string().into_boxed_str()),
            port.parse::<u16>().unwrap_or(3450),
        ),
        password,
    )
    .await;

    let mut client = match client {
        Ok(c) => {
            println!(
                "  {} Connected to {}:{}\n",
                "✓".green().bold(),
                hostname.white().bold(),
                port.white().bold()
            );
            c
        }
        Err(e) => {
            print_err(&format!("Connection failed: {:?}", e));
            exit(1);
        }
    };

    let prompt = format!("{}:{} → ", hostname.bright_cyan(), port.bright_cyan());
    let mut rl = Editor::<()>::new();

    loop {
        let cmd = rl.readline(&prompt);
        match cmd {
            Ok(cmd) => {
                if cmd == "exit" || cmd == "quit" {
                    println!("\n  {} Bye! 👋\n", "✓".green().bold());
                    exit(0);
                }
                if cmd.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(&cmd);
                let res = execute(&cmd, &mut client).await;
                let is_docs = cmd
                    .split_whitespace()
                    .next()
                    .map(|s| s.to_uppercase() == "DOCS")
                    .unwrap_or(false);

                if is_docs && res.0 == NetPacketState::OK {
                    print_docs(&res.1);
                } else if res.0 == NetPacketState::ERR {
                    print_err(&res.1);
                } else {
                    smart_format(&res.1);
                }
                println!();
            }
            Err(_) => {
                println!("\n  {} Bye! 👋\n", "✓".green().bold());
                exit(0);
            }
        }
    }
}

pub async fn execute(command: &str, client: &mut DoreaClient) -> (NetPacketState, String) {
    let mut parts: Vec<&str> = command.split_whitespace().collect();
    let op = parts.remove(0);

    match op.to_uppercase().as_str() {
        "GET" => {
            if parts.len() != 1 {
                return (NetPacketState::ERR, "Usage: GET <key>".into());
            }
            let key = parts[0];
            // 直接发送命令给服务器
            let command = format!("get {}", key);
            match client.execute(&command).await {
                Ok((state, data)) => {
                    if state == NetPacketState::OK {
                        let result = String::from_utf8_lossy(&data).to_string();
                        // 如果服务器返回空字符串，可能是 key 不存在
                        if result.is_empty() {
                            (NetPacketState::ERR, "Key not found".into())
                        } else {
                            (state, result)
                        }
                    } else {
                        (state, String::from_utf8_lossy(&data).to_string())
                    }
                }
                Err(e) => (NetPacketState::ERR, e.to_string()),
            }
        }
        "SET" => {
            // 直接发送原始命令给服务器，不分割重组，保留引号结构
            match client.execute(command).await {
                Ok((state, data)) => {
                    let result = String::from_utf8_lossy(&data).to_string();
                    (state, result)
                }
                Err(e) => (NetPacketState::ERR, e.to_string()),
            }
        }
        "SETEX" => {
            if parts.len() < 3 {
                return (
                    NetPacketState::ERR,
                    "Usage: SETEX <key> <value> <expire>".into(),
                );
            }
            let key = parts[0];
            let expire = parts.last().unwrap().parse::<usize>().unwrap_or(0);
            let value = parts[1..parts.len() - 1].join(" ");
            match client.setex(key, DataValue::from(&value), expire).await {
                Ok(_) => (NetPacketState::OK, "".into()),
                Err(e) => (NetPacketState::ERR, e.to_string()),
            }
        }
        "BINARY" => {
            if parts.len() < 2 {
                return (
                    NetPacketState::ERR,
                    "Usage: BINARY <stringify|tovec|download|upload> <key> ...".into(),
                );
            }
            let sub = parts[0];
            let key = parts[1];

            match sub.to_uppercase().as_str() {
                "STRINGIFY" => match client.get(key).await {
                    Some(DataValue::Binary(bin)) => (
                        NetPacketState::OK,
                        String::from_utf8(bin.read()).unwrap_or_default(),
                    ),
                    Some(v) => (NetPacketState::OK, v.to_string()),
                    None => (NetPacketState::ERR, "Key not found".into()),
                },
                "TOVEC" => match client.get(key).await {
                    Some(DataValue::Binary(bin)) => {
                        (NetPacketState::OK, format!("{:?}", bin.read()))
                    }
                    Some(v) => (NetPacketState::OK, v.to_string()),
                    None => (NetPacketState::ERR, "Key not found".into()),
                },
                "DOWNLOAD" => {
                    if parts.len() != 3 {
                        return (
                            NetPacketState::ERR,
                            "Usage: BINARY DOWNLOAD <key> <filename>".into(),
                        );
                    }
                    match client.get(key).await {
                        Some(DataValue::Binary(bin)) => {
                            let mut path = dirs::download_dir().unwrap();
                            path.push(parts[2]);
                            std::fs::File::create(&path)
                                .unwrap()
                                .write_all(&bin.read())
                                .unwrap();
                            (NetPacketState::OK, format!("Saved to {:?}", path))
                        }
                        Some(v) => (NetPacketState::OK, v.to_string()),
                        None => (NetPacketState::ERR, "Key not found".into()),
                    }
                }
                "UPLOAD" => {
                    if parts.len() != 3 {
                        return (
                            NetPacketState::ERR,
                            "Usage: BINARY UPLOAD <key> <filepath>".into(),
                        );
                    }
                    let path = PathBuf::from(parts[2]);
                    if !path.is_file() {
                        return (NetPacketState::ERR, "File not found".into());
                    }
                    let mut buf = vec![];
                    std::fs::File::open(&path)
                        .unwrap()
                        .read_to_end(&mut buf)
                        .unwrap();
                    match client
                        .setex(key, DataValue::Binary(Binary::build(buf)), 0)
                        .await
                    {
                        Ok(_) => (NetPacketState::OK, "".into()),
                        Err(_) => (NetPacketState::ERR, "Upload failed".into()),
                    }
                }
                _ => (NetPacketState::ERR, format!("Unknown operation: {}", sub)),
            }
        }
        _ => match client.execute(command).await {
            Ok(p) => (p.0, String::from_utf8(p.1).unwrap_or_default()),
            Err(e) => (NetPacketState::ERR, e.to_string()),
        },
    }
}
