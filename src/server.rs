use std::{fs, path::PathBuf};

use tokio::net::TcpListener;
use tokio::task;

use crate::config::DoreaFileConfig;
use crate::handle;

pub struct DoreaServer {
    server_options: ServerOption,
    server_listener: TcpListener,
    server_config: DoreaFileConfig,

    connection_number: u16,
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

        let config = crate::config::load_config(&document_path).unwrap();

        let listner = match TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                panic!("Server startup error: {}", e);
            },
        };

        Self {
            server_options: options,
            server_listener: listner,
            server_config: config,
            connection_number: 0,
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

            task::spawn(async move {
                let _ = handle::process(&mut socket, config, current_db).await;
            });


        }

    }

}

#[cfg(test)]
mod server_test {
    #[tokio::test]
    async fn try_to_bind() {

        println!("Docuemnt Path: {:?}",dirs::data_local_dir());

        let mut dorea = crate::server::DoreaServer::bind(crate::server::ServerOption {
            hostname: "0.0.0.0",
            port: 3450,
            document_path: None,
            quiet_runtime: true,
        }).await;

        dorea.listen().await;
    }
}