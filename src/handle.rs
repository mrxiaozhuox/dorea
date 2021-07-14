use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::command::CommandManager;
use crate::configure::DoreaFileConfig;
use crate::database::DataBaseManager;
use crate::network::{Frame, NetPacket, NetPacketState};
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

    let mut frame = Frame::new();

    let mut message: Vec<u8>;

    loop {

        message = match frame.parse_frame(socket).await {
            Ok(message) => message,
            Err(e) => { 

                if e.to_string() == "Connection reset by peer (os error 54)" {
                    return Err(e);
                }

                NetPacket::make(
                    e.to_string().as_bytes().to_vec(),
                    NetPacketState::ERR
                ).send(socket).await?;
                continue;
            },
        };

        if message.len() == 0 { continue; }

        let res = CommandManager
            ::command_handle(
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
