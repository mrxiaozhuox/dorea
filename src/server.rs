use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{task, sync::Mutex};
use tokio::sync::OnceCell;
use tokio::time::Interval;

use crate::Result;
use crate::handle;
use crate::database::{DataBaseManager};
use once_cell::sync::Lazy;
use toml::Value;
use std::fmt::Formatter;
use std::time::Duration;

#[derive(Debug)]
struct ListenerOptions {
    hostname: String,
    port: u16,
    max_connection: u16,
}

pub struct Listener {
    listener: TcpListener,
    options: ListenerOptions,
}

static DB_MANAGER: Lazy<Mutex<DataBaseManager>> = Lazy::new(|| {
    let m = DataBaseManager::new();
    Mutex::new(m)
});

static DB_CONFIG: OnceCell<Value> = OnceCell::const_new();

async fn config_bind() -> Value { DB_MANAGER.lock().await.init() }

impl Listener {
    // construct a new Listener
    pub async fn new(hostname:&str, port: u16) -> Listener {

        // init database config
        DB_CONFIG.get_or_init(config_bind).await;

        let mut index = 1;
        // while index <= 130 {
        //     DB_MANAGER.lock().await.insert(format!("_{}",index),crate::database::DataValue::String(String::from("world")),String::from("default"),None);
        //     index += 1;
        // }

        println!("{:?}",DB_MANAGER.lock().await);

        let addr = format!("{}:{}", hostname, port);
        let app = match TcpListener::bind(&addr).await {
            Ok(t) => t,
            Err(e) => {
                panic!("Server startup error: {}", e);
            }
        };

        let option = ListenerOptions {
            hostname: hostname.to_string(),
            port,
            max_connection: 255,
        };

        Listener {
            listener: app,
            options: option,
        }
    }

    pub async fn start(&mut self) {

        let config = DB_CONFIG.get().unwrap();

        let schedule = config["memory"].get("persistence_interval");
        let schedule = schedule.unwrap().as_integer().unwrap();

        // persistence task
        task::spawn(async move {
            loop {
                DB_MANAGER.lock().await.persistence_all();
                tokio::time::sleep(Duration::from_millis(schedule as u64)).await;
            }
        });

        loop {
            let (mut socket, socket_addr ) = self.listener.accept().await.unwrap();

            // a new connect was created.
            println!("A new connection was created: @{:?}",socket_addr);

            task::spawn(async move {
                loop {
                    match process(&mut socket).await {
                        Ok(text) => {
                            let text: String = "+".to_string() + &text + "\n";
                            socket.write_all((text).as_ref()).await.unwrap();
                        },
                        Err(e) => {
                            // display database error.
                            if e != "empty string" {
                                let text: String = "-".to_string() + &e + "\n";
                                let _ = socket.write_all((text).as_ref()).await;
                            }
                        },
                    };
                }
            });
        }
    }
}

async fn process(socket: &mut TcpStream) -> Result<String> {
    let mut buf = [0;1024];

    // get data buffer size
    let length = match socket.read(&mut buf).await {
        Ok(t) => t,
        Err(_e) => 0,
    };

    // if length eq zero, abort the function
    if length == 0 { return Err("unknown input".to_string()) }

    let mut split: usize = length;

    if buf[length - 1] == 10 { split = length - 1 } // for Linux & MacOS
    else if buf[length - 2] == 13 && buf[length - 1] == 10 { split = length - 2 } // for Windows

    // from buf[u8; 1024] to String
    let message = String::from_utf8_lossy(&buf[0 .. split]).to_string();

    let parse_result = handle::parser(message);
    let parse_meta: handle::ParseMeta;

    if parse_result.is_err() {
        let err = match parse_result.err() {
            Some(err) => err,
            None => "unknown error".to_string(),
        };

        return Err(err);
    }else{
        parse_meta = parse_result.unwrap();
    }

    let exec_result = handle::execute(&DB_MANAGER,parse_meta).await;

    return match exec_result {
        Ok(res) => {
            Ok(res)
        }
        Err(err) => {
            Err(err)
        }
    }
}