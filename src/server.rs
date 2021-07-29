use std::sync::Arc;
use std::{fs, path::PathBuf};

use log::{error, info};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task;

use crate::configure::DoreaFileConfig;
use crate::database::DataBaseManager;
use crate::handle;

pub struct DoreaServer {
    _server_options: ServerOption,
    server_listener: TcpListener,
    server_config: DoreaFileConfig,
    startup_time: i64,

    connection_number: Arc<Mutex<ConnectNumber>>,
    db_manager: Arc<Mutex<DataBaseManager>>,
}

pub struct ServerOption {
    pub hostname: &'static str,
    pub port: u16,
    pub document_path: Option<PathBuf>,
    pub quiet_runtime: bool,
}

impl DoreaServer {
    pub async fn bind(options: ServerOption) -> Self {
        let document_path = match &options.document_path {
            Some(buf) => buf.clone(),
            None => {
                let temp = dirs::data_local_dir().unwrap();
                temp.join("Dorea")
            }
        };

        if !document_path.is_dir() {
            fs::create_dir_all(&document_path).unwrap();
        }

        let options: ServerOption = ServerOption {
            hostname: options.hostname,
            port: options.port,
            document_path: Some(document_path.clone()),
            quiet_runtime: options.quiet_runtime,
        };

        let addr = format!("{}:{}", options.hostname, options.port);

        // try to load logger system
        crate::logger::init_logger().expect("logger init failed");

        let config = crate::configure::load_config(&document_path).unwrap();

        info!("configure loaded from: {:?}", document_path);

        let listener = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                panic!("Server startup error: {}", e);
            }
        };

        Self {
            _server_options: options,
            server_listener: listener,
            server_config: config.clone(),
            connection_number: Arc::new(Mutex::new(ConnectNumber { num: 0 })),
            db_manager: Arc::new(Mutex::new(DataBaseManager::new(document_path.clone()))),
            startup_time: chrono::Local::now().timestamp() + 100,
        }
    }

    pub async fn listen(&mut self) {
        info!("dorea is running, ready to accept connections.");

        loop {
            // wait for client connect.
            let (mut socket, socket_addr) = match self.server_listener.accept().await {
                Ok(value) => value,
                Err(_) => {
                    continue;
                }
            };

            if self.connection_number.lock().await.get()
                >= self.server_config.connection.max_connect_number
            {
                drop(socket);
                error!(
                    "exceeded max connections number: {}.",
                    self.connection_number.lock().await.get()
                );
                continue;
            }

            info!("new client connected: '{:?}'.", socket_addr);

            // add connection number (+1).
            self.connection_number.lock().await.add();

            let config = self.server_config.clone();

            let current_db = config.database.default_group.to_string();

            let current = current_db.clone();

            let db_manager = Arc::clone(&self.db_manager);

            let connect_num = Arc::clone(&self.connection_number);

            task::spawn(async move {
                let _ = handle::process(&mut socket, config, current, &db_manager).await;

                // connection number -1;
                connect_num.lock().await.low();
            });
        }
    }
}

struct ConnectNumber {
    num: u16,
}

impl ConnectNumber {
    pub fn add(&mut self) {
        self.num += 1;
    }
    pub fn low(&mut self) {
        self.num -= 1;
    }
    pub fn get(&self) -> u16 {
        self.num
    }
}
