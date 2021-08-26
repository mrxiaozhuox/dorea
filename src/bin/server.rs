use dorea::server::{DoreaServer, ServerOption};
use dorea::DOREA_VERSION;
use clap::{App, Arg};

use std::path::PathBuf;

const TEMPLATE: &'static str = "
⎐ {bin} - (V{version}) ⎐

USAGE:
  {usage}

FLAGS:
{flags}

OPTIONS:
{options}

Dorea-Core: https://github.com/doreadb/dorea.git
Dorea-Repo: https://github.com/doreadb/
Author: ZhuoEr Liu <mrxzx.info@gmail.com>
";

#[tokio::main]
async fn main() {

    let matches = App::new("Dorea Server")
        .version(DOREA_VERSION)
        .author("ZhuoEr Liu <mrxzx@qq.com>")
        .about("Dorea storage system")
        .arg(
            Arg::with_name("HOSTNAME")
                .short("h")
                .long("hostname")
                .default_value("127.0.0.1")
                .display_order(0)
        )
        .arg(
            Arg::with_name("PORT")
                .short("p")
                .long("port")
                .default_value("3450")
                .display_order(1)
        )
        .arg(
            Arg::with_name("WORKSPACE")
                .long("workspace")
                .default_value("$DOREA_DOC")
                .display_order(2)
        )
        .arg(
            Arg::with_name("LOGLEVEL")
                .long("loglevel")
                .default_value("INFO")
                .display_order(3)
        )
        .template(TEMPLATE)
        .get_matches();

    let hostname = matches.value_of("HOSTNAME").unwrap().to_string();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap_or(3450);
    let workspace = matches.value_of("WORKSPACE").unwrap();
    let log_level = matches.value_of("LOGLEVEL").unwrap();

    let workspace: Option<PathBuf> = match workspace {
        "$DOREA_DOC" => None,
        other => { Some(PathBuf::from(other)) }
    };

    // 输出标志！经典QWQ
    println!(
        "
        _____     ____    _____    ______
        |  __ \\   / __ \\  |  __ \\  |  ____|     /\\
        | |  | | | |  | | | |__) | | |__       /  \\
        | |  | | | |  | | |  _  /  |  __|     / /\\ \\
        | |__| | | |__| | | | \\ \\  | |____   / ____ \\
        |_____/   \\____/  |_|  \\_\\ |______| /_/    \\_\\

        「 Dorea:{} 」server address: {}
        ",
        crate::DOREA_VERSION,
        format!("dorea://{}:{}",hostname,port)
    );

    // 生成服务器实例
    let mut server = DoreaServer::bind(ServerOption {
        hostname: Box::leak(hostname.into_boxed_str()),
        port: 3450,
        document_path: workspace.clone(),
        logger_level: log_level.into()
    }).await;

    server.listen().await;
}