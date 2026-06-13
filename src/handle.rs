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
        // 读取请求
        let message = match frame.parse_frame(socket).await {
            Ok(message) => message,
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
        };

        if message.is_empty() {
            return Ok(());
        }

        // 检查是否是 Pipeline 模式
        if frame.latest_state == NetPacketState::PIPELINE {
            // Pipeline 模式：body 是命令数量 (u32)
            let count = u32::from_be_bytes([
                message.get(0).copied().unwrap_or(0),
                message.get(1).copied().unwrap_or(0),
                message.get(2).copied().unwrap_or(0),
                message.get(3).copied().unwrap_or(0),
            ]) as usize;

            let count = count.min(10000); // 限制最大数量

            // 批量读取所有命令
            let mut requests: Vec<Vec<u8>> = Vec::with_capacity(count);
            for _ in 0..count {
                match frame.parse_frame(socket).await {
                    Ok(msg) => {
                        if msg.is_empty() {
                            break;
                        }
                        requests.push(msg);
                    }
                    Err(_) => break,
                }
            }

            // 批量处理
            let responses = process_batch(
                &requests,
                &mut auth,
                &mut current,
                &mut value_ser_style,
                &config,
                &database_manager,
                &connect_id,
                startup_time,
            ).await;

            // 批量发送响应
            let buffer = serialize_batch_responses(&responses);
            socket.write_all(&buffer).await?;
        } else {
            // 普通模式：处理单个请求
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
            let body = process_response_body(&res.1, startup_time, &database_manager, &config).await;

            NetPacket::make(body, res.0).send(socket).await?;
        }
    }
}

/// 批量处理请求
async fn process_batch(
    requests: &[Vec<u8>],
    auth: &mut bool,
    current: &mut String,
    value_ser_style: &mut String,
    config: &DoreaFileConfig,
    database_manager: &Arc<DataBaseManager>,
    connect_id: &uuid::Uuid,
    startup_time: i64,
) -> Vec<(Vec<u8>, NetPacketState)> {
    let mut responses = Vec::with_capacity(requests.len());

    for message in requests {
        let res = CommandManager::command_handle(
            String::from_utf8_lossy(&message[..]).to_string(),
            auth,
            current,
            value_ser_style,
            config,
            database_manager,
            connect_id,
        )
        .await;

        if res.0 == NetPacketState::EMPTY {
            continue;
        }

        let body = process_response_body(&res.1, startup_time, database_manager, config).await;
        responses.push((body, res.0));
    }

    responses
}

/// 处理响应 body（处理预设数据）
async fn process_response_body(
    res_body: &[u8],
    startup_time: i64,
    database_manager: &Arc<DataBaseManager>,
    config: &DoreaFileConfig,
) -> Vec<u8> {
    match String::from_utf8_lossy(res_body).to_string().as_str() {
        "@[SERVER_STARTUP_TIME]" => startup_time.to_string().as_bytes().to_vec(),
        val if val.starts_with("@[PRELOAD_DB]:") => {
            let db_name = val[14..].to_string();
            let tmp_db_manager = database_manager.clone();
            let db_config = config.database.clone();
            tokio::spawn(async move {
                crate::database::DB_STATE
                    .lock()
                    .await
                    .insert(db_name.clone(), crate::database::DataBaseState::LOADING);

                let storage_path = tmp_db_manager.location.clone().join("storage");

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
            vec![]
        }
        _ => res_body.to_vec(),
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
