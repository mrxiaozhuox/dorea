use dorea::server::{DoreaServer, ServerOption};
use dorea::DOREA_VERSION;
use clap::{App, Arg};

use std::path::PathBuf;

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
        )
        .arg(
            Arg::with_name("PORT")
                .short("p")
                .long("port")
                .default_value("3450")
        )
        .arg(
            Arg::with_name("WORKSPACE")
                .short("w")
                .long("workspace")
                .default_value("$DOREA_DOCUMENT_PATH")
        )
        .get_matches();

    let hostname = matches.value_of("HOSTNAME").unwrap().to_string();
    let port = matches.value_of("PORT").unwrap().parse::<u16>().unwrap_or(3450);
    let workspace = matches.value_of("WORKSPACE").unwrap();

    let workspace: Option<PathBuf> = match workspace {
        "$DOREA_DOCUMENT_PATH" => None,
        other => { Some(PathBuf::from(other)) }
    };

    println!(
        "
        _____     ____    _____    ______
        |  __ \\   / __ \\  |  __ \\  |  ____|     /\\
        | |  | | | |  | | | |__) | | |__       /  \\
        | |  | | | |  | | |  _  /  |  __|     / /\\ \\
        | |__| | | |__| | | | \\ \\  | |____   / ____ \\
        |_____/   \\____/  |_|  \\_\\ |______| /_/    \\_\\

        「 Dorea:{} 」service address: {}
        ",
        crate::DOREA_VERSION,
        format!("dorea://{}:{}",hostname,port)
    );

    let mut server = DoreaServer::bind(ServerOption {
        hostname: Box::leak(hostname.into_boxed_str()),
        port: 3450,
        document_path: workspace,
        quiet_runtime: false
    }).await;

    server.listen().await;
}