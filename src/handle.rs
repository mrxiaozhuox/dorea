use tokio::net::TcpStream;

use crate::config::DoreaFileConfig;

pub(crate) async fn process(
    socket: TcpStream, 
    config: DoreaFileConfig,
    current: String,
) {
    let readonly: bool;

    // check readonly state
    if config.database.readonly_group.contains(&current) {
        readonly = true;
    } else {
        readonly = false;
    }


}