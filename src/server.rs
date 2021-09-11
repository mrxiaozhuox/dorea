use std::sync::Arc;
use std::{fs, path::PathBuf};

use log::{error, info};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task;

use crate::configure::DoreaFileConfig;
use crate::database::DataBaseManager;
use crate::event::EventManager;
use crate::handle;
use crate::plugin::PluginManager;

use once_cell::sync::Lazy;

// 判断服务器是否已被初始化过
static INIT_STATE: Lazy<Mutex<InitState>> = Lazy::new(|| Mutex::new(InitState { state: false }));

struct InitState { state: bool }

pub struct DoreaServer {
    _server_options: ServerOption,
    server_listener: TcpListener,
    server_config: DoreaFileConfig,
    plugin_manager: PluginManager,
    startup_time: i64,

    connection_number: Arc<Mutex<ConnectNumber>>,
    db_manager: Arc<Mutex<DataBaseManager>>,
}

pub struct ServerOption {
    pub hostname: &'static str,
    pub port: u16,
    pub document_path: Option<PathBuf>,
    pub logger_level: String,
}

impl DoreaServer {
    pub async fn bind(options: ServerOption) -> Self {

        // 检查服务器对象在同一程序中是否被多次创建
        if INIT_STATE.lock().await.state {
            panic!("Server objects can only be created once!");
        } else {
            INIT_STATE.lock().await.state = true;
        }

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
            logger_level: options.logger_level,
        };

        let addr = format!("{}:{}", options.hostname, options.port);

        // try to load logger system
        crate::logger::init_logger(&options.logger_level.to_uppercase()).expect("logger init failed");

        let config = crate::configure::load_config(&document_path).unwrap();

        info!("configure loaded from: {:?}", document_path);

        let listener = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                panic!("Server startup error: {}", e);
            }
        };

        let plugins = PluginManager::init(&document_path).unwrap();

        let object = Self {
            _server_options: options,
            server_listener: listener,
            server_config: config.clone(),
            plugin_manager: todo!(),
            connection_number: Arc::new(Mutex::new(ConnectNumber { num: 0 })),
            db_manager: Arc::new(Mutex::new(DataBaseManager::new(document_path.clone()))),
            startup_time: chrono::Local::now().timestamp() + 100,
        };

        // -- 其他线程服务初始代码 --

        // 事件驱动器加载
        let event_manager = EventManager::init(object.db_manager.clone()).await;

        tokio::task::spawn(async move {
            event_manager.loop_events().await;
        });

        // 返回本体对象

        object
    }

    pub async fn listen(&mut self) {

        info!("dorea is running, ready to accept connections.");

        let doc_path = self._server_options.document_path.clone().unwrap();

        let _ = crate::service::startup(
            (self._server_options.hostname.clone(), self._server_options.port),
            &doc_path
        ).await;

        loop {
            // wait for client connect.
            let (mut socket, _) = match self.server_listener.accept().await {
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

            // info!("new client connected: '{:?}'.", socket_addr);

            // add connection number (+1).
            self.connection_number.lock().await.add();

            let config = self.server_config.clone();

            let current_db = config.database.default_group.to_string();

            let current = current_db.clone();

            let db_manager = Arc::clone(&self.db_manager);

            let connect_num = Arc::clone(&self.connection_number);

            let startup_time = self.startup_time;

            task::spawn(async move {
                let _ = handle::process(
                    &mut socket,
                    config,
                    current,
                    &db_manager,
                    startup_time,
                ).await;

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
