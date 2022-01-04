use std::sync::Arc;

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
    database_manager: Arc<Mutex<DataBaseManager>>,
    startup_time: i64,
    value_ser_style: String,
    connect_id: uuid::Uuid,
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
                &database_manager,
                &connect_id,
            )
            .await;

        if res.0 != NetPacketState::EMPTY {

            // 将预设的数据转换为数据本值
            let body = match String::from_utf8_lossy(&res.1[..]).to_string().as_str() {
                "@[SERVER_STARTUP_TIME]" => {
                    startup_time.to_string().as_bytes().to_vec()
                },
                _ => {

                    let mut tmp = res.1.clone();

                    let val = String::from_utf8_lossy(&res.1[..]).to_string();
                    if val.len() > 14 && &val[0..14] == "@[PRELOAD_DB]:" {
                        let db_name = String::from(&val[14..]);
                        let tmp_db_manager = database_manager.clone();
                        tokio::spawn(async move {

                            crate::database::DB_STATE.lock().await.insert(
                                db_name.clone(), 
                                crate::database::DataBaseState::LOADING
                            );

                            let storage_path = tmp_db_manager.lock().await.location.clone().join("storage");
                            let db_config = tmp_db_manager.lock().await.config.database.clone();

                            let ndb = crate::database::DataBase::init(
                                db_name.to_string(),
                                storage_path,
                                db_config,
                            ).await;

                            match tmp_db_manager.lock().await.load_from(&db_name, ndb).await {
                                Ok(_) => {
                                    crate::database::DB_STATE.lock().await.insert(
                                        db_name.clone(), 
                                        crate::database::DataBaseState::NORMAL
                                    );
                                },
                                Err(e) => {
                                    log::error!("database load error: {}", e.to_string())
                                }
                            };
                        });
                        tmp = vec![];
                    }

                    tmp
                } /* 不是预设数据，不处理 */
            };

            NetPacket::make(body, res.0).send(socket).await?;
        } else {
            // if is empty: connection closed
            return Ok(());
        }

    }
}