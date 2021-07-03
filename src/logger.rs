use std::{fs, path::PathBuf};


pub async fn init_logger(path: String) {

    let root = PathBuf::from(&path).join("log");

    if !root.is_dir() {
        fs::create_dir(&root).unwrap();
    }

    let file_path = PathBuf::from(&root).join("config.yaml");

    // init logger config
    if !file_path.is_file() {
        let response = reqwest::get(
            "https://raw.githubusercontent.com/doreadb/dorea/master/config/logger.yaml"
        ).await;

        let response = match response {
            Ok(v) => v,
            Err(_) => {
                reqwest::get(
                    "https://gitee.com/mrxzx/dorea/raw/master/config/logger.yaml"
                ).await.unwrap()
            },
        };

        let text = response.text().await.unwrap();

        let text = text.replace(":DOREA_LOG_PATH:", root.to_str().unwrap());

        match fs::write(&file_path, text) {
            Ok(_) => { /* continue */ },
            Err(e) => panic!("{}", e),
        }
    }

    log4rs::init_file(&file_path,Default::default()).unwrap();
}