use std::sync::Arc;
use std::{fs, path::PathBuf};

use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task;

use crate::configuration::DoreaFileConfig;
use crate::database::DataBaseManager;
use crate::handle;

pub struct DoreaServer {
    _server_options: ServerOption,
    server_listener: TcpListener,
    server_config: DoreaFileConfig,

    connection_number: u16,
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

        let document_path = match &options.document_path{
            Some(buf) => buf.clone(),
            None => {
                let temp = dirs::data_local_dir().unwrap();
                temp.join("Dorea")
            },
        };

        if ! document_path.is_dir() {
            fs::create_dir_all(&document_path).unwrap();
        }

        let options: ServerOption = ServerOption {
            hostname: options.hostname,
            port: options.port,
            document_path: Some(document_path.clone()),
            quiet_runtime: options.quiet_runtime,
        };

        let addr = format!("{}:{}",options.hostname, options.port);

        let config = crate::configuration::load_config(&document_path).unwrap();

        let listner = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                panic!("Server startup error: {}", e);
            },
        };

        Self {
            _server_options: options,
            server_listener: listner,
            server_config: config.clone(),
            connection_number: 0,
            db_manager: Arc::new(
                Mutex::new(DataBaseManager::new(
                    document_path.clone(),
                ))
            )
        }
    }

    pub async fn listen(&mut self) {

        loop {
            
            // wait for client connect.
            let (mut socket, _ ) = match self.server_listener.accept().await {
                Ok(value) => value,
                Err(_) => { continue; },
            };

            // add connection number (+1).
            self.connection_number += 1;

            let config = self.server_config.clone();

            let current_db = config.database.default_group.to_string();

            let current = current_db.clone();

            let db_manager = Arc::clone(&self.db_manager);

            task::spawn(async move {

                let _ = handle::process(
                    &mut socket, 
                    config, 
                    current,
                    &db_manager
                ).await;

            });

        }

    }
}