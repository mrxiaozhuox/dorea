use clap::clap_app;
use dorea::client::{DoreaClient};
use rustyline::Editor;
use dorea::network::NetPacketState;
use dorea::value::DataValue;
use anyhow::Error;
use std::future::Future;
use std::process::exit;

#[tokio::main]
pub async fn main() {

    let matches = clap_app!(dorea =>
        (version: "0.2.1")
        (author: "ZhuoEr Liu <mrxzx@qq.com>")
        (about: "Does awesome things")
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
            Ok(v) => { (NetPacketState::OK, v.to_string()) }
            Err(e) => { (NetPacketState::ERR, e.to_string()) }
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