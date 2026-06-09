use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use crate::command::CommandManager;
use crate::configure::DoreaFileConfig;
use crate::database::{DataBase, DataBaseManager};
use crate::network::{Frame, NetPacket, NetPacketState, MAGIC, PROTOCOL_VERSION};
use crate::Result;

// connection process
pub(crate) async fn process(
    socket: &mut TcpStream,
    config: DoreaFileConfig,
    current: String,
    database_manager: Arc<DataBaseManager>,
    startup_time: i64,
    value_ser_style: String,
    connect_id: uuid::Uuid,
) -> Result<()> {
    let mut current = current;
    let mut value_ser_style = value_ser_style;

    let mut auth = false;

    if config.connection.connection_password.is_empty() {
        auth = true;
    }

    let mut frame = Frame::new();

    loop {
        // 批量读取请求
        let mut requests: Vec<Vec<u8>> = Vec::new();

        // 读取第一个请求（阻塞）
        match frame.parse_frame(socket).await {
            Ok(message) => {
                if message.is_empty() {
                    return Ok(());
                }
                requests.push(message);
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("invalid magic") || err_msg.contains("unsupported protocol") {
                    return Err(e);
                }
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    if io_err.kind() == std::io::ErrorKind::ConnectionReset
                        || io_err.kind() == std::io::ErrorKind::UnexpectedEof
                    {
                        return Err(e);
                    }
                }

                NetPacket::make(e.to_string().as_bytes().to_vec(), NetPacketState::ERR)
                    .send(socket)
                    .await?;
                continue;
            }
        }

        // 尝试读取更多请求（带超时，用于 pipeline 优化）
        socket.set_nodelay(true)?;
        loop {
            // 检查是否有更多数据可读（带 1ms 超时）
            let mut peek_buf = [0u8; 1];
            let peek_result = tokio::time::timeout(
                std::time::Duration::from_millis(1),
                socket.peek(&mut peek_buf)
            ).await;

            match peek_result {
                Ok(Ok(0)) => break, // 没有更多数据
                Ok(Ok(_)) => {
                    // 有数据，尝试读取一个完整的帧
                    match frame.parse_frame(socket).await {
                        Ok(msg) => {
                            if msg.is_empty() {
                                break;
                            }
                            requests.push(msg);
                        }
                        Err(_) => break, // 读取失败，停止批量读取
                    }
                }
                Ok(Err(_)) => break,
                Err(_) => break, // 超时，说明没有更多数据
            }

            // 防止无限循环，限制单次批量处理的请求数
            if requests.len() >= 1000 {
                break;
            }
        }

        // 批量处理请求
        let mut responses: Vec<(Vec<u8>, NetPacketState)> = Vec::with_capacity(requests.len());

        for message in requests {
            let res = CommandManager::command_handle(
                String::from_utf8_lossy(&message[..]).to_string(),
                &mut auth,
                &mut current,
                &mut value_ser_style,
                &config,
                &database_manager,
                &connect_id,
            )
            .await;

            if res.0 == NetPacketState::EMPTY {
                return Ok(());
            }

            // 将预设的数据转换为数据本值
            let body = match String::from_utf8_lossy(&res.1[..]).to_string().as_str() {
                "@[SERVER_STARTUP_TIME]" => startup_time.to_string().as_bytes().to_vec(),
                _ => {
                    let mut tmp = res.1.clone();

                    let val = String::from_utf8_lossy(&res.1[..]).to_string();
                    if val.len() > 14 && &val[0..14] == "@[PRELOAD_DB]:" {
                        let db_name = String::from(&val[14..]);
                        let tmp_db_manager = database_manager.clone();
                        let db_config = config.database.clone();
                        tokio::spawn(async move {
                            crate::database::DB_STATE
                                .lock()
                                .await
                                .insert(db_name.clone(), crate::database::DataBaseState::LOADING);

                            let storage_path =
                                tmp_db_manager.location.clone().join("storage");

                            let ndb = DataBase::init(
                                db_name.to_string(),
                                storage_path,
                                db_config,
                            )
                            .await;

                            match tmp_db_manager.load_from(&db_name, Arc::new(RwLock::new(ndb))).await {
                                Ok(_) => {
                                    crate::database::DB_STATE.lock().await.insert(
                                        db_name.clone(),
                                        crate::database::DataBaseState::NORMAL,
                                    );
                                }
                                Err(e) => {
                                    log::error!("database load error: {}", e.to_string())
                                }
                            };
                        });
                        tmp = vec![];
                    }

                    tmp
                }
            };

            responses.push((body, res.0));
        }

        // 批量发送响应
        if responses.len() == 1 {
            // 单个响应，直接发送
            NetPacket::make(responses[0].0.clone(), responses[0].1)
                .send(socket)
                .await?;
        } else if responses.len() > 1 {
            // 多个响应，批量序列化后一次性发送
            let buffer = serialize_batch_responses(&responses);
            socket.write_all(&buffer).await?;
        }
    }
}

/// 批量序列化多个响应到一个 buffer
fn serialize_batch_responses(responses: &[(Vec<u8>, NetPacketState)]) -> Vec<u8> {
    let mut buffer = Vec::new();
    for (body, state) in responses {
        // MAGIC
        buffer.extend_from_slice(&MAGIC);
        // VERSION + STATE
        buffer.push(PROTOCOL_VERSION);
        buffer.push(state.to_u8());
        // LENGTH (4 bytes, big endian)
        buffer.extend_from_slice(&(body.len() as u32).to_be_bytes());
        // BODY
        buffer.extend_from_slice(body);
    }
    buffer
}
