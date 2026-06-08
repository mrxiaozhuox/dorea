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

/// 打印启动 Banner
fn print_banner() {
    println!();
    println!(
        "{}",
        r#"
    ╭──────────────────────────────────────╮
    │   ____                              │
    │  |  _ \  ___  _ __  ___  ___        │
    │  | | | |/ _ \| '_ \/ __|/ _ \       │
    │  | |_| | (_) | | | \__ \  __/       │
    │  |____/ \___/|_| |_|___/\___|       │
    │                                      │
    │  A Key-Value Storage System          │
    ╰──────────────────────────────────────╯
    "#
        .bright_cyan()
    );
    println!("  {} {}", "Version:".dimmed(), "v0.5.0".green());
    println!(
        "  {} {}",
        "Docs:".dimmed(),
        "Type 'docs' to see available commands".yellow()
    );
    println!();
}

/// 打印帮助提示
fn print_help_hint() {
    println!(
        "  💡 Type {} to see available commands\n",
        "'docs'".yellow().bold()
    );
}

/// 格式化 JSON 输出（带语法高亮）
fn format_json_output(json_str: &str) -> String {
    // 尝试解析并美化 JSON
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_str) {
        let formatted = serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| json_str.to_string());
        // 为 JSON 添加颜色
        let mut result = String::new();
        for line in formatted.lines() {
            if line.contains(':') {
                // 键值对
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].bright_blue();
                    let value = highlight_json_value(parts[1].trim());
                    result.push_str(&format!("{}:{}\n", key, value));
                    continue;
                }
            }
            result.push_str(&format!("{}\n", line));
        }
        result.trim_end().to_string()
    } else {
        json_str.to_string()
    }
}

/// 高亮 JSON 值
fn highlight_json_value(value: &str) -> String {
    let trimmed = value.trim_end_matches(',');
    let comma = if value.ends_with(',') { "," } else { "" };
    
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        // 字符串 - 绿色
        format!("{}{}", trimmed.green(), comma)
    } else if trimmed == "true" || trimmed == "false" {
        // 布尔值 - 黄色
        format!("{}{}", trimmed.yellow(), comma)
    } else if trimmed == "null" {
        // null - 灰色
        format!("{}{}", trimmed.dimmed(), comma)
    } else if trimmed.parse::<f64>().is_ok() {
        // 数字 - 青色
        format!("{}{}", trimmed.cyan(), comma)
    } else {
        format!("{}{}", trimmed, comma)
    }
}

/// 格式化数组输出
fn format_array_output(arr: &[String]) -> String {
    if arr.is_empty() {
        return "[]".dimmed().to_string();
    }
    
    let mut result = String::from("[\n");
    for item in arr {
        result.push_str(&format!("  {} {}\n", "•".bright_cyan(), item.white()));
    }
    result.push(']');
    result
}

/// 打印成功消息
fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// 打印错误消息
fn print_error(message: &str) {
    println!("{} {}", "✗".red().bold(), message);
}

/// 打印信息消息
fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// 打印数据结果
fn print_data(data: &str) {
    // 尝试解析为 JSON
    if data.starts_with('{') || data.starts_with('[') {
        println!("{}", format_json_output(data));
    } else if data.starts_with('[') && data.ends_with(']') {
        // 可能是数组字符串格式
        println!("{}", format_json_output(data));
    } else {
        // 普通字符串，加引号显示
        println!("{} {}", "→".bright_cyan(), data.white());
    }
}

/// 打印 docs 帮助文档（美化）
fn print_docs(content: &str) {
    println!();
    println!("{}", "┌─────────────────────────────────────────────────────┐".bright_cyan());
    println!("{}", "│               📚 Dorea Command Reference            │".bright_cyan());
    println!("{}", "└─────────────────────────────────────────────────────┘".bright_cyan());
    println!();
    
    // 解析并美化文档内容
    let lines: Vec<&str> = content.lines().collect();
    let mut in_code_block = false;
    let mut current_section = String::new();
    
    for line in lines {
        let trimmed = line.trim();
        
        // 检测标题（命令名称）
        if trimmed.starts_with('#') && !trimmed.starts_with("##") {
            current_section = trimmed.trim_start_matches('#').trim().to_string();
            println!();
            println!("  {} {}", "▸".bright_cyan().bold(), current_section.white().bold());
            println!("  {}", "─".repeat(50).dimmed());
        } else if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                println!();
            }
        } else if in_code_block {
            // 代码块内容
            println!("    {}", trimmed.bright_black().italic());
        } else if !trimmed.is_empty() {
            // 普通文本
            println!("  {}", trimmed.dimmed());
        }
    }
    
    println!();
    println!("{}", "─".repeat(55).dimmed());
    println!(
        "  {} Type {} for a specific command",
        "💡".yellow(),
        "'docs <command>'".yellow().bold()
    );
    println!();
}

#[tokio::main]
pub async fn main() {
    let matches = App::new("Dorea Cli")
        .version("0.5.0")
        .author("YuKun Liu <mrxzx.info@gmail.com>")
        .about("DoreaDB Cli Tool")
        .arg(
            Arg::with_name("HOSTNAME")
                .short("h")
                .long("hostname")
                .takes_value(true)
                .help("Set the server hostname")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::with_name("PORT")
                .short("p")
                .long("port")
                .takes_value(true)
                .help("Set the server port")
                .default_value("3450"),
        )
        .arg(
            Arg::with_name("PASSWORD")
                .short("a")
                .long("password")
                .takes_value(true)
                .help("Connect password")
                .default_value(""),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Try to run a command with dorea-cli")
                .arg(Arg::with_name("COMMAND").required(true).index(1))
                .arg(
                    Arg::with_name("HOSTNAME")
                        .short("h")
                        .long("hostname")
                        .takes_value(true)
                        .help("Set the server hostname")
                        .default_value("127.0.0.1"),
                )
                .arg(
                    Arg::with_name("PORT")
                        .short("p")
                        .long("port")
                        .takes_value(true)
                        .help("Set the server port")
                        .default_value("3450"),
                )
                .arg(
                    Arg::with_name("PASSWORD")
                        .short("a")
                        .long("password")
                        .takes_value(true)
                        .help("Connect password")
                        .default_value(""),
                )
                .arg(
                    Arg::with_name("DATABASE")
                        .short("t")
                        .long("database")
                        .takes_value(true)
                        .help("Target database")
                        .default_value("default"),
                ),
        )
        .get_matches();

    let hostname = matches.value_of("HOSTNAME").unwrap();
    let port = matches.value_of("PORT").unwrap();
    let password = matches.value_of("PASSWORD").unwrap();

    // 单次执行模式
    if let Some(matches) = matches.subcommand_matches("run") {
        let hostname = matches.value_of("HOSTNAME").unwrap();
        let port = matches.value_of("PORT").unwrap();
        let password = matches.value_of("PASSWORD").unwrap();
        let target = matches.value_of("DATABASE").unwrap();
        let command = matches.value_of("COMMAND").unwrap();

        let tc = DoreaClient::connect(
            (
                Box::leak(hostname.to_string().into_boxed_str()),
                port.parse::<u16>().unwrap_or(3450),
            ),
            password,
        )
        .await;

        let mut tc = match tc {
            Ok(c) => c,
            Err(err) => {
                print_error(&format!("Connection failed: {:?}", err));
                exit(1);
            }
        };

        tc.select(target).await.expect("database select failed!");

        let res = execute(command, &mut tc).await;
        if res.0 == NetPacketState::ERR {
            print_error(&res.1);
        } else {
            print_data(&res.1);
        }

        return;
    }

    // 交互模式
    print_banner();

    let c = DoreaClient::connect(
        (
            Box::leak(hostname.to_string().into_boxed_str()),
            port.parse::<u16>().unwrap_or(3450),
        ),
        password,
    )
    .await;

    let mut c = match c {
        Ok(c) => {
            println!(
                "  {} Connected to {}:{}\n",
                "✓".green().bold(),
                hostname.white().bold(),
                port.white().bold()
            );
            print_help_hint();
            c
        }
        Err(err) => {
            print_error(&format!("Connection failed: {:?}", err));
            exit(1);
        }
    };

    let prompt = format!("{}:{}→ ", hostname.bright_cyan(), port.bright_cyan());
    let mut readline = Editor::<()>::new();

    loop {
        let cmd = readline.readline(&prompt);
        match cmd {
            Ok(cmd) => {
                if cmd == "exit" || cmd == "quit" {
                    println!("\n  {} Goodbye! 👋\n", "✓".green().bold());
                    exit(0)
                }
                if cmd.is_empty() {
                    continue;
                }

                // 添加到历史记录
                let _ = readline.add_history_entry(&cmd);

                let res = execute(&cmd, &mut c).await;
                let cmd_parts: Vec<&str> = cmd.split_whitespace().collect();
                let is_docs = cmd_parts.first().map(|s| s.to_uppercase() == "DOCS").unwrap_or(false);

                if is_docs {
                    if res.0 == NetPacketState::OK {
                        print_docs(&res.1);
                    } else {
                        print_error("Document load failed");
                    }
                } else if res.0 == NetPacketState::ERR {
                    print_error(&res.1);
                } else if res.0 == NetPacketState::NOAUTH {
                    print_error("Authentication required. Use 'auth <password>' to login.");
                } else {
                    // 成功
                    if res.1.is_empty() {
                        print_success("OK");
                    } else {
                        print_data(&res.1);
                    }
                }
                
                println!(); // 添加空行分隔
            }
            Err(_) => {
                println!("\n  {} Goodbye! 👋\n", "✓".green().bold());
                std::process::exit(0);
            }
        }
    }
}

// cli 命令运行
pub async fn execute(command: &str, client: &mut DoreaClient) -> (NetPacketState, String) {
    let mut slice: Vec<&str> = command.split_whitespace().collect();
    let operation = slice.remove(0);

    if operation.to_uppercase() == "GET" {
        if slice.len() != 1 {
            return (
                NetPacketState::ERR,
                "Usage: GET <key>".to_string(),
            );
        }

        return match client.get(slice.first().unwrap()).await {
            Some(v) => {
                let output = v.to_string();
                // 尝试美化 JSON
                if output.starts_with('{') || output.starts_with('[') {
                    (NetPacketState::OK, output)
                } else {
                    (NetPacketState::OK, output)
                }
            }
            None => (NetPacketState::ERR, "Key not found".to_string()),
        };
    } else if operation.to_uppercase() == "SET" {
        if slice.len() < 2 {
            return (
                NetPacketState::ERR,
                "Usage: SET <key> <value> [expire]".to_string(),
            );
        }

        let mut temp = slice.clone();
        let key = temp.remove(0);
        temp.retain(|x| !x.trim().is_empty());

        let value: String = temp.join(" ");
        let value = DataValue::from(&value);

        return match client.setex(key, value, 0).await {
            Ok(_) => (NetPacketState::OK, "".to_string()),
            Err(e) => (NetPacketState::ERR, e.to_string()),
        };
    } else if operation.to_uppercase() == "SETEX" {
        if slice.len() < 3 {
            return (
                NetPacketState::ERR,
                "Usage: SETEX <key> <value> <expire_seconds>".to_string(),
            );
        }

        let mut temp = slice.clone();
        let key = temp.remove(0);
        let expire = temp.last().unwrap();
        let expire = match expire.parse::<usize>() {
            Ok(v) => {
                temp.remove(temp.len() - 1);
                v
            }
            Err(_) => 0,
        };

        temp.retain(|x| !x.trim().is_empty());
        let value: String = temp.join(" ");
        let value = DataValue::from(&value);

        return match client.setex(key, value, expire).await {
            Ok(_) => (NetPacketState::OK, "".to_string()),
            Err(e) => (NetPacketState::ERR, e.to_string()),
        };
    } else if operation.to_uppercase() == "BINARY" {
        if slice.len() < 2 {
            return (
                NetPacketState::ERR,
                "Usage: BINARY <stringify|tovec|download|upload> <key> [args...]".to_string(),
            );
        }

        let sub = slice.first().unwrap();
        let key = slice.get(1).unwrap();

        if sub.to_uppercase() == "STRINGIFY" {
            return match client.get(key).await {
                Some(v) => {
                    if let DataValue::Binary(bin) = v {
                        let bytes = bin.read();
                        return (
                            NetPacketState::OK,
                            String::from_utf8(bytes).unwrap_or_default(),
                        );
                    }
                    return (NetPacketState::OK, v.to_string());
                }
                None => (NetPacketState::ERR, "Key not found".to_string()),
            };
        } else if sub.to_uppercase() == "TOVEC" {
            return match client.get(key).await {
                Some(v) => {
                    if let DataValue::Binary(bin) = v {
                        let bytes = bin.read();
                        return (NetPacketState::OK, format!("{:?}", bytes));
                    }
                    return (NetPacketState::OK, v.to_string());
                }
                None => (NetPacketState::ERR, "Key not found".to_string()),
            };
        } else if sub.to_uppercase() == "DOWNLOAD" {
            if slice.len() != 3 {
                return (
                    NetPacketState::ERR,
                    "Usage: BINARY DOWNLOAD <key> <filename>".to_string(),
                );
            }

            let filename = slice.get(2).unwrap();

            return match client.get(key).await {
                Some(v) => {
                    if let DataValue::Binary(bin) = v {
                        let bytes = bin.read();
                        let mut download_dir = dirs::download_dir().unwrap();
                        download_dir.push(filename);

                        let mut file = std::fs::File::create(&download_dir).unwrap();
                        file.write_all(&bytes[..]).unwrap();

                        return (NetPacketState::OK, format!("Saved to {:?}", download_dir));
                    }
                    return (NetPacketState::OK, v.to_string());
                }
                None => (NetPacketState::ERR, "Key not found".to_string()),
            };
        } else if sub.to_uppercase() == "UPLOAD" {
            if slice.len() != 3 {
                return (
                    NetPacketState::ERR,
                    "Usage: BINARY UPLOAD <key> <filepath>".to_string(),
                );
            }

            let filename = slice.get(2).unwrap();
            let path = PathBuf::from(filename);

            if !path.is_file() {
                return (NetPacketState::ERR, "File not found".to_string());
            }

            let mut file = std::fs::File::open(path).unwrap();
            let mut buf = vec![];
            file.read_to_end(&mut buf).unwrap();

            match client
                .setex(key, DataValue::Binary(Binary::build(buf.clone())), 0)
                .await
            {
                Ok(_) => return (NetPacketState::OK, "".to_string()),
                Err(_) => return (NetPacketState::ERR, "Upload failed".to_string()),
            };
        }

        return (NetPacketState::ERR, format!("Unknown binary operation: {}", sub));
    }

    let res = client.execute(command).await;
    match res {
        Ok(p) => {
            let mut message = String::from_utf8(p.1).unwrap_or_default();
            if message.is_empty() {
                message = "".to_string();
            }
            (p.0, message)
        }
        Err(err) => (NetPacketState::ERR, err.to_string()),
    }
}
