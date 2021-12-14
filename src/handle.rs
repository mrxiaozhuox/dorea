use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::command::CommandManager;
use crate::configure::DoreaFileConfig;
use crate::database::DataBaseManager;
use crate::network::{Frame, NetPacket, NetPacketState};
use crate::Result;
use crate::plugin::PluginManager;

// connection process
pub(crate) async fn process(
    socket: &mut TcpStream,
    config: DoreaFileConfig,
    current: String,
    database_manager: &Mutex<DataBaseManager>,
    plugin_manager: &Mutex<PluginManager>,
    startup_time: i64,
    value_ser_style: String,
) -> Result<()> {

    let mut current = current;
    let mut value_ser_style = value_ser_style;

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

        if message.len() == 0 { return Ok(()); }

        let res = CommandManager
            ::command_handle(
                String::from_utf8_lossy(&message[..]).to_string(),
                &mut auth,
                &mut current,
                &mut value_ser_style,
                &config,
                database_manager,
                plugin_manager,
            )
            .await;

        if res.0 != NetPacketState::EMPTY {

            // 将预设的数据转换为数据本值
            let body = match String::from_utf8_lossy(&res.1[..]).to_string().as_str() {
                "@[SERVER_STARTUP_TIME]" => {
                    startup_time.to_string().as_bytes().to_vec()
                }
                _ => { res.1 } /* 不是预设数据，不处理 */
            };

            NetPacket::make(body, res.0).send(socket).await?;
        } else {
            // if is empty: connection closed
            return Ok(());
        }

    }
}