//! Dorea server implementation
//!
//! you can use this code to start a server.
//! ```rust
//! use dorea::server::Listener;
//!
//! let mut listener = Listener::new("127.0.0.1",3450).await;
//! listener.start().await;
//! ```
//! then the tcpServer will run in { hostname : 127.0.0.1, port : 3450 }
//!
//! you can use `nc` tool to connect it ( **and dorea-client is better** )

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::{task, sync::Mutex};
use tokio::sync::OnceCell;
use crate::Result;
use crate::handle;
use crate::database::{DataBaseManager};
use once_cell::sync::Lazy;
use toml::Value;
use std::time::Duration;

#[derive(Debug)]
struct ListenerOptions {
    hostname: String,
    port: u16,
    connection_number: u16,
}

pub struct Listener {
    listener: TcpListener,
    options: ListenerOptions,
}

struct ConnectNumber {
    num: u16,
}

static DB_MANAGER: Lazy<Mutex<DataBaseManager>> = Lazy::new(|| {
    let m = DataBaseManager::new();
    Mutex::new(m)
});

static CONNECT_NUM: Lazy<Mutex<ConnectNumber>> = Lazy::new(|| {
    Mutex::new(ConnectNumber { num: 0 })
});

static DB_CONFIG: OnceCell<Value> = OnceCell::const_new();

async fn config_bind() -> Value { DB_MANAGER.lock().await.init() }

/// the Listener can help you to create a new Dorea server.
impl Listener {

    /// structure a new listener struct.
    pub async fn new(hostname:&str, port: u16) -> Listener {

        // init database config
        DB_CONFIG.get_or_init(config_bind).await;

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
            connection_number: 0,
        };

        Listener {
            listener: app,
            options: option,
        }
    }

    /// start the Dorea server.
    /// **Note**: you need use `.await` for this function.
    pub async fn start(&mut self) {

        let config = DB_CONFIG.get().unwrap();

        let schedule = config["memory"].get("persistence_interval");
        let schedule = schedule.unwrap().as_integer().unwrap();

        let max_connect = match config["common"].get("maximum_connect_number") {
            None => 98,
            Some(v) => { v.as_integer().unwrap() as u16 }
        };

        // persistence task
         task::spawn(async move {
            loop {
                DB_MANAGER.lock().await.persistence_all();
                tokio::time::sleep(Duration::from_millis(schedule as u64)).await;
            }
        });

        loop {
            let (mut socket, socket_addr ) = self.listener
                .accept().await.unwrap();

            // a new connect was created.
            println!("A new connection was created: @{:?}",socket_addr);

            let connect_num: u16 = CONNECT_NUM.lock().await.get();
            println!("{}:{}",connect_num.clone(),max_connect.clone());
            if connect_num >= max_connect {
                let _ = socket.write_all(("-connection error\n").as_bytes()).await;
                continue;
            }


            CONNECT_NUM.lock().await.add();
            task::spawn(async move {
                loop {
                    match process(&mut socket).await {
                        Ok(text) => {
                            let text: String = "+".to_string() + &text + "\n";
                            let res = socket.write_all((text).as_ref()).await;

                            if res.is_err() {
                                CONNECT_NUM.lock().await.low();
                                break;
                            }
                        },
                        Err(e) => {

                            // display database error.
                            if e != "empty string" {
                                let text: String = "-".to_string() + &e + "\n";
                                let res = socket.write_all((text).as_ref()).await;

                                if res.is_err() {
                                    CONNECT_NUM.lock().await.low();
                                    break;
                                }
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

impl ConnectNumber {
    pub fn add(&mut self) { self.num += 1; }
    pub fn low(&mut self) { self.num -= 1; }
    pub fn get(&self) -> u16 { self.num }
}