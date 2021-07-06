use std::{fs, path::PathBuf};

pub struct DoreaServer {
    server_options: ServerOption
}

pub struct ServerOption {
    pub hostname: &'static str,
    pub port: u16,
    pub document_path: Option<PathBuf>,
    pub quiet_runtime: bool,
}

impl DoreaServer {

    pub fn bind(options: ServerOption) -> Self {

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

        crate::config::load_config(&document_path).unwrap();

        let options: ServerOption = ServerOption {
            hostname: options.hostname,
            port: options.port,
            document_path: Some(document_path),
            quiet_runtime: options.quiet_runtime,
        };

        Self {
            server_options: options
        }
    }

}

#[cfg(test)]
mod server_test {
    #[test]
    fn try_to_bind() {
        println!("{:?}",dirs::data_local_dir().unwrap());
        let _dorea = crate::server::DoreaServer::bind(crate::server::ServerOption {
            hostname: "127.0.0.1",
            port: 3450,
            document_path: None,
            quiet_runtime: true,
        });
    }
}