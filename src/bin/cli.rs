use clap::clap_app;
use dorea::client::{DoreaClient};
use rustyline::Editor;
use dorea::network::NetPacketState;

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
                let res = execute(&cmd, &mut c).await;
                println!("[{:?}]: {}",res.0, res.1);
            }
            Err(_) => { std::process::exit(0); } /* exit cli system */
        }
    }
}

// cli 命令运行
pub async fn execute(command: &str, client: &mut DoreaClient) -> (NetPacketState, String) {
    let res = client.execute(&command).await;
    return match res {
        Ok(p) => {
            (p.0, String::from_utf8(p.1).unwrap_or(String::new()))
        }
        Err(err) => { (NetPacketState::ERR ,err.to_string()) }
    }
}