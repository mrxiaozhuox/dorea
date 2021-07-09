use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::command::CommandManager;
use crate::configuration::DoreaFileConfig;
use crate::database::DataBaseManager;
use crate::network::{self, NetPacket, NetPacketState};
use crate::Result;

// connection process
pub(crate) async fn process(
    socket: &mut TcpStream,
    config: DoreaFileConfig,
    current: String,
    database_manager: &Mutex<DataBaseManager>,
) -> Result<()> {
    let mut current = current;

    let mut auth = false;

    if config.connection.connection_password == "" {
        auth = true;
    }

    let _readonly: bool;

    // check readonly state
    if config.database.readonly_group.contains(&current) {
        _readonly = true;
    } else {
        _readonly = false;
    }

    let mut command_manager = CommandManager::new();

    let mut message: Vec<u8>;

    loop {
        message = network::parse_frame(socket).await;

        let res = command_manager
            .command_handle(
                String::from_utf8_lossy(&message[..]).to_string(),
                &mut auth,
                &mut current,
                &config,
                database_manager,
            )
            .await;

        if res.0 != NetPacketState::EMPTY {
            NetPacket::make(res.1, res.0).send(socket).await?;
        } else {
            // if is empty: connection closed
            return Ok(());
        }
    }
}
