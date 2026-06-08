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
    println!("{}", r#"
 /$$$$$$$                                          
| $$__  $$                                         
| $$  \ $$  /$$$$$$   /$$$$$$   /$$$$$$   /$$$$$$ 
| $$  | $$ /$$__  $$ /$$__  $$ /$$__  $$ |____  $$
| $$  | $$| $$  \ $$| $$  \__/| $$$$$$$$  /$$$$$$$
| $$  | $$| $$  | $$| $$      | $$_____/ /$$__  $$
| $$$$$$$/|  $$$$$$/| $$      |  $$$$$$$|  $$$$$$$
|_______/  \______/ |__/       \_______/ \_______/
                                                  
  A Key-Value Storage System
                                                  
"#.bright_cyan());
    println!("  {} {}", "Version:".dimmed(), DOREA_CLI_VERSION.green());
    println!("  {} {}", "Hint:".dimmed(), "Type 'docs' to see available commands".yellow());
    println!();
}

/// 格式化 JSON 输出（带语法高亮）
fn format_json(json_str: &str) -> String {
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Ok(pretty) = serde_json::to_string_pretty(&json_value) {
            return highlight_json(&pretty);
        }
    }
    json_str.to_string()
}

/// JSON 语法高亮
fn highlight_json(json: &str) -> String {
    let mut result = String::new();
    for line in json.lines() {
        if line.contains(':') {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].bright_blue().to_string();
                let value = highlight_json_value(parts[1]);
                result.push_str(&format!("{}:{}\n", key, value));
                continue;
            }
        }
        result.push_str(&format!("{}\n", line));
    }
    result.trim_end().to_string()
}

/// 高亮 JSON 值
fn highlight_json_value(value: &str) -> String {
    let trimmed = value.trim_end_matches(',');
    let comma = if value.ends_with(',') { "," } else { "" };
    
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        format!("{}{}", trimmed.green(), comma)
    } else if trimmed == "true" || trimmed == "false" {
        format!("{}{}", trimmed.yellow(), comma)
    } else if trimmed == "null" {
        format!("{}{}", trimmed.dimmed(), comma)
    } else if trimmed.parse::<f64>().is_ok() {
        format!("{}{}", trimmed.cyan(), comma)
    } else {
        format!("{}{}", trimmed, comma)
    }
}

/// 打印成功
fn print_ok(msg: &str) {
    if msg.is_empty() {
        println!("{} {}", "✓".green().bold(), "OK".white());
    } else if msg.starts_with('{') || msg.starts_with('[') {
        println!("{}", format_json(msg));
    } else {
        println!("{} {}", "→".bright_cyan(), msg.white());
    }
}

/// 打印错误
fn print_err(msg: &str) {
    println!("{} {}", "✗".red().bold(), msg.red());
}

/// 打印 docs 文档
fn print_docs(content: &str) {
    println!();
    println!("{}", "┌─────────────────────────────────────────────────────┐".bright_cyan());
    println!("{}", "│               📚 Dorea Command Docs                 │".bright_cyan());
    println!("{}", "└─────────────────────────────────────────────────────┘".bright_cyan());
    println!();
    
    let mut in_code = false;
    for line in content.lines() {
        let t = line.trim();
        
        if t.starts_with('#') && !t.starts_with("##") {
            println!();
            println!("  {} {}", "▸".bright_cyan(), t.trim_start_matches('#').trim().white().bold());
            println!("  {}", "─".repeat(50).dimmed());
        } else if t.starts_with("```") {
            in_code = !in_code;
            if in_code { println!(); }
        } else if in_code {
            println!("    {}", t.bright_black().italic());
        } else if !t.is_empty() {
            println!("  {}", t.dimmed());
        }
    }
    
    println!();
    println!("{}", "─".repeat(55).dimmed());
    println!("  {} Try {} for specific command", "💡".yellow(), "'docs <command>'".yellow().bold());
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
                .arg(Arg::with_name("HOSTNAME").short("h").long("hostname").takes_value(true).default_value("127.0.0.1"))
                .arg(Arg::with_name("PORT").short("p").long("port").takes_value(true).default_value("3450"))
                .arg(Arg::with_name("PASSWORD").short("a").long("password").takes_value(true).default_value(""))
                .arg(Arg::with_name("DATABASE").short("t").long("database").takes_value(true).default_value("default")),
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
            (Box::leak(hostname.to_string().into_boxed_str()), port.parse::<u16>().unwrap_or(3450)),
            password,
        ).await;

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
            print_ok(&res.1);
        }
        return;
    }

    // 交互模式
    print_banner();

    let hostname = matches.value_of("HOSTNAME").unwrap();
    let port = matches.value_of("PORT").unwrap();
    let password = matches.value_of("PASSWORD").unwrap();

    let client = DoreaClient::connect(
        (Box::leak(hostname.to_string().into_boxed_str()), port.parse::<u16>().unwrap_or(3450)),
        password,
    ).await;

    let mut client = match client {
        Ok(c) => {
            println!("  {} Connected to {}:{}\n", "✓".green().bold(), hostname.white().bold(), port.white().bold());
            c
        }
        Err(e) => {
            print_err(&format!("Connection failed: {:?}", e));
            exit(1);
        }
    };

    let prompt = format!("{}:{}→ ", hostname.bright_cyan(), port.bright_cyan());
    let mut rl = Editor::<()>::new();

    loop {
        let cmd = rl.readline(&prompt);
        match cmd {
            Ok(cmd) => {
                if cmd == "exit" || cmd == "quit" {
                    println!("\n  {} Bye! 👋\n", "✓".green().bold());
                    exit(0);
                }
                if cmd.is_empty() { continue; }
                
                let _ = rl.add_history_entry(&cmd);
                let res = execute(&cmd, &mut client).await;
                let is_docs = cmd.split_whitespace().next().map(|s| s.to_uppercase() == "DOCS").unwrap_or(false);

                if is_docs && res.0 == NetPacketState::OK {
                    print_docs(&res.1);
                } else if res.0 == NetPacketState::ERR {
                    print_err(&res.1);
                } else {
                    print_ok(&res.1);
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
            match client.get(parts[0]).await {
                Some(v) => (NetPacketState::OK, v.to_string()),
                None => (NetPacketState::ERR, "Key not found".into()),
            }
        }
        "SET" => {
            if parts.len() < 2 {
                return (NetPacketState::ERR, "Usage: SET <key> <value>".into());
            }
            let key = parts[0];
            let value = parts[1..].join(" ");
            match client.setex(key, DataValue::from(&value), 0).await {
                Ok(_) => (NetPacketState::OK, "".into()),
                Err(e) => (NetPacketState::ERR, e.to_string()),
            }
        }
        "SETEX" => {
            if parts.len() < 3 {
                return (NetPacketState::ERR, "Usage: SETEX <key> <value> <expire>".into());
            }
            let key = parts[0];
            let expire = parts.last().unwrap().parse::<usize>().unwrap_or(0);
            let value = parts[1..parts.len()-1].join(" ");
            match client.setex(key, DataValue::from(&value), expire).await {
                Ok(_) => (NetPacketState::OK, "".into()),
                Err(e) => (NetPacketState::ERR, e.to_string()),
            }
        }
        "BINARY" => {
            if parts.len() < 2 {
                return (NetPacketState::ERR, "Usage: BINARY <stringify|tovec|download|upload> <key> ...".into());
            }
            let sub = parts[0];
            let key = parts[1];

            match sub.to_uppercase().as_str() {
                "STRINGIFY" => match client.get(key).await {
                    Some(DataValue::Binary(bin)) => (NetPacketState::OK, String::from_utf8(bin.read()).unwrap_or_default()),
                    Some(v) => (NetPacketState::OK, v.to_string()),
                    None => (NetPacketState::ERR, "Key not found".into()),
                },
                "TOVEC" => match client.get(key).await {
                    Some(DataValue::Binary(bin)) => (NetPacketState::OK, format!("{:?}", bin.read())),
                    Some(v) => (NetPacketState::OK, v.to_string()),
                    None => (NetPacketState::ERR, "Key not found".into()),
                },
                "DOWNLOAD" => {
                    if parts.len() != 3 {
                        return (NetPacketState::ERR, "Usage: BINARY DOWNLOAD <key> <filename>".into());
                    }
                    match client.get(key).await {
                        Some(DataValue::Binary(bin)) => {
                            let mut path = dirs::download_dir().unwrap();
                            path.push(parts[2]);
                            std::fs::File::create(&path).unwrap().write_all(&bin.read()).unwrap();
                            (NetPacketState::OK, format!("Saved to {:?}", path))
                        }
                        Some(v) => (NetPacketState::OK, v.to_string()),
                        None => (NetPacketState::ERR, "Key not found".into()),
                    }
                }
                "UPLOAD" => {
                    if parts.len() != 3 {
                        return (NetPacketState::ERR, "Usage: BINARY UPLOAD <key> <filepath>".into());
                    }
                    let path = PathBuf::from(parts[2]);
                    if !path.is_file() {
                        return (NetPacketState::ERR, "File not found".into());
                    }
                    let mut buf = vec![];
                    std::fs::File::open(&path).unwrap().read_to_end(&mut buf).unwrap();
                    match client.setex(key, DataValue::Binary(Binary::build(buf)), 0).await {
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
