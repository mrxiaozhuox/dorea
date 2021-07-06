use std::path::PathBuf;

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
            None => PathBuf::new(),
        };

        crate::config::load_config(document_path).unwrap();

        Self {
            server_options: options
        }
    }

}

#[cfg(test)]
mod server_test {
    #[test]
    fn try_to_bind() {
        crate::server::DoreaServer::bind(crate::server::ServerOption {
            hostname: "127.0.0.1",
            port: 3450,
            document_path: Some(std::path::PathBuf::from("./dorea/")),
            quiet_runtime: true,
        });
    }
}