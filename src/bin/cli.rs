//! Dorea-CLI
//! Author: YuKun Liu<mrxzx.info@gmail.com>
//! Date: 2021/10/25
//! @DoreaDB Client

use clap::clap_app;
use dorea::client::{DoreaClient};
use doson::binary::Binary;
use rustyline::Editor;
use dorea::network::NetPacketState;
use dorea::value::DataValue;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

#[tokio::main]
pub async fn main() {

    let matches = clap_app!(dorea =>
        (version: "0.2.1")
        (author: "ZhuoEr Liu <mrxzx@qq.com>")
        (about: "DoreaDB Cli Tool")
        (@arg HOSTNAME: -h --hostname +takes_value "Set the server hostname")
        (@arg PORT: -p --port +takes_value "Set the server port")
        (@arg PASSWORD: -a --password +takes_value "Connect password")
    ).get_matches();

    let hostname = match matches.value_of("HOSTNAME") {
        None => "127.0.0.1",
        Some(v) => v
    }.to_string();

    let port = match matches.value_of("PORT") {
        None => 3450,
        Some(v) => {
            match v.parse::<u16>() {
                Ok(n) => n,
                Err(_) => 3450
            }
        }
    };

    let password = match matches.value_of("PASSWORD") {
        None => "",
        Some(v) => v
    };

    let password = password.clone();

    // 获取数据库客户端连接
    let c =DoreaClient::connect(
        (
            Box::leak(hostname.clone().into_boxed_str()),
            port
        ),
        password
    ).await;

    let mut c = match c {
        Ok(c) => c,
        Err(err) => {
            panic!("{:?}", err);
        }
    };

    let prompt = format!("{}:{} ~> ",hostname,port);
    let mut readline = Editor::<()>::new();

    loop {
        let cmd = readline.readline(&prompt);
        match cmd {
            Ok(cmd) => {

                if cmd == "exit" { exit(0) }
                if cmd == "" { continue }

                let res = execute(&cmd, &mut c).await;
                println!("[{:?}]: {}",res.0, res.1);
            }
            Err(_) => { std::process::exit(0); } /* exit cli system */
        }
    }
}

// cli 命令运行
pub async fn execute(command: &str, client: &mut DoreaClient) -> (NetPacketState, String) {

    // 判断操作类型
    let mut slice: Vec<&str> = command.split(" ").collect();
    let operation = slice.remove(0);

    if operation.to_uppercase() == "GET" {

        if slice.len() != 1 {
            return (NetPacketState::ERR, "Incorrect number of parameters".to_string());
        }

        return match client.get(slice.get(0).unwrap()).await {
            Some(v) => { (NetPacketState::OK, v.to_string()) }
            None => { (NetPacketState::ERR, "Value not found.".to_string()) }
        }

    }else if operation.to_uppercase() == "SET" {

        if slice.len() < 2 {
            return (NetPacketState::ERR, "Missing command parameters.".to_string());
        }

        let mut temp = slice.clone();

        let key = temp.remove(0);

        temp.retain(|x| { x.trim() != "" });


        let value: String = temp.join(" ");
        let value = DataValue::from(&value);

        return match client.setex(key, value, 0).await {
            Ok(_) => {
                (NetPacketState::OK, "Successful.".to_string())
            }
            Err(e) => {
                (NetPacketState::ERR, e.to_string())
            }
        };

    } else if operation.to_uppercase() == "SETEX" {

        if slice.len() < 3 {
            return (NetPacketState::ERR, "Missing command parameters.".to_string());
        }

        let mut temp = slice.clone();

        let key = temp.remove(0);

        let expire = temp.get(temp.len() - 1).unwrap();
        let expire = match expire.parse::<usize>() {
            Ok(v) => {
                temp.remove(temp.len() - 1);
                v
            },
            Err(_) => 0
        };

        temp.retain(|x| { x.trim() != "" });

        let value: String = temp.join(" ");
        let value = DataValue::from(&value);

        return match client.setex(key, value, expire).await {
            Ok(_) => {
                (NetPacketState::OK, "Successful.".to_string())
            }
            Err(e) => {
                (NetPacketState::ERR, e.to_string())
            }
        };

    } else if operation.to_uppercase() == "BINARY" {
        
        // 二进制操作功能
        // 目前 Dorea 内部没有提供 Binary 原生函数
        // Cli 专门封装了 Binary 的一些基本用途函数
        // 二进制转字符串、二进制下载、二进制上传、二进制数组等

        if slice.len() < 2 {
            return (NetPacketState::ERR, "Missing command parameters.".to_string());
        }

        // 子命令和目标 key
        let sub = slice.get(0).unwrap();
        let key = slice.get(1).unwrap();

        if sub.to_uppercase() == "STRINGIFY" {
            return match client.get(&key).await {
                Some(v) => {
                    if let DataValue::Binary(bin) = v {
                        let bytes = bin.read();
                        return (NetPacketState::OK, String::from_utf8(bytes).unwrap_or(String::new()));
                    }
                    return (NetPacketState::OK, v.to_string());
                }
                None => { (NetPacketState::ERR, "Value not found.".to_string()) }
            }
        } else if sub.to_uppercase() == "TOVEC" {
            return match client.get(&key).await {
                Some(v) => {
                    if let DataValue::Binary(bin) = v {
                        let bytes = bin.read();
                        return (NetPacketState::OK, format!("{:?}", bytes));
                    }
                    return (NetPacketState::OK, v.to_string());
                }
                None => { (NetPacketState::ERR, "Value not found.".to_string()) }
            }   
        } else if sub.to_uppercase() == "DOWNLOAD" {

            if slice.len() != 3 {
                return (NetPacketState::ERR, "Missing command parameters.".to_string());
            }

            let filename = slice.get(2).unwrap();

            return match client.get(&key).await {
                Some(v) => {
                    if let DataValue::Binary(bin) = v {
                        
                        let bytes = bin.read();

                        let mut download_dir = dirs::download_dir().unwrap();
                        download_dir.push(filename);

                        let mut file = std::fs::File::create(&download_dir).unwrap();

                        file.write_all(&bytes[..]).unwrap();

                        return (NetPacketState::OK, format!("{:?}", download_dir));
                    }
                    return (NetPacketState::OK, v.to_string());
                }
                None => { (NetPacketState::ERR, "Value not found.".to_string()) }
            }   
        } else if sub.to_uppercase() == "UPLOAD" {
         
            if slice.len() != 3 {
                return (NetPacketState::ERR, "Missing command parameters.".to_string());
            }

            let filename = slice.get(2).unwrap(); 

            let path = PathBuf::from(filename);

            if ! path.is_file() {
                return (NetPacketState::ERR, "Path not a file.".to_string())
            }

            let mut file = std::fs::File::open(path).unwrap();
            
            let mut buf = vec![];

            file.read_to_end(&mut buf).unwrap();

            match client.setex(key, DataValue::Binary(Binary::build(buf.clone())), 0).await {
                Ok(_) => {return (NetPacketState::OK, "Successful.".to_string()) }
                Err(_) => { return (NetPacketState::ERR, "Upload failed.".to_string()) }   
            };
        }

        return (NetPacketState::ERR, "Unknown sub-operation.".to_string());
    }

    let res = client.execute(&command).await;
    return match res {
        Ok(p) => {

            let mut message = String::from_utf8(p.1).unwrap_or(String::new());

            if message == "" {
                message = "Successful.".to_string();
            }

            (p.0, message)
        }
        Err(err) => { (NetPacketState::ERR ,err.to_string()) }
    }
}