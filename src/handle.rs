use bytes::{BufMut,BytesMut};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::database::DataBaseManager;
use crate::{configuration::DoreaFileConfig};
use crate::network::{NetPacket, NetPacketState};
use crate::command::CommandManager;
use crate::Result;


// connection process
pub(crate) async fn process(
    socket: &mut TcpStream, 
    config: DoreaFileConfig,
    current: String,
    database_manager: &Mutex<DataBaseManager>
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

    let mut buffer = [0; 2048];
    let mut message = BytesMut::with_capacity(2048);

    loop {

        let size = socket.read(&mut buffer).await?;
        message.put(&buffer[0..size]);

        let res = command_manager.command_handle(
            String::from_utf8_lossy(&buffer[0..size]).to_string(),
            &mut auth,
            &mut current,
            &config,
            database_manager,
        );

        if res.0 != NetPacketState::EMPTY {
            NetPacket::make(res.1, res.0).send(socket).await?;
        }

    }
}