//!
//! 关于 Web-Service API 接口结构声明
//!
//! ```json
//! {
//!     "alpha": "OK",
//!     "data": {},
//!     "message": ""
//! }
//! ```
//!
//! alpha   主要用于快速检查请求状况：类似于 `dorea-protocol` 中的 `status`.
//! data    主要数据项：所有数据结果包含在里面
//! message 错误信息（仅在错误时有内容）

// axum 0.4
use axum::extract::ws::Message;
use axum::extract::{self, TypedHeader, WebSocketUpgrade};
use axum::response::{Json, Response};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::client::{DoreaClient, InfoType};
use crate::network::NetPacketState;
use crate::service::secret;
use crate::service::ShareState;
use crate::value::DataValue;
use axum::http::StatusCode;

use super::db;

// Dorea Web 主页
pub async fn index() -> Api {
    Api::json(
        StatusCode::OK,
        json!(format!("dorea: V{}", crate::DOREA_VERSION)),
    )
}

// 授权系统（JWT 发放）控制函数
pub async fn auth(
    form: extract::Form<crate::service::secret::SecretForm>,
    state: extract::Extension<Arc<ShareState>>,
) -> Api {
    let username = form.username.clone();
    let password = form.password.clone();

    let mut account_info = db::ServiceAccountInfo {
        usable: false,
        username: username.clone(),
        password: password.clone(),
        usa_database: None,
        cls_command: vec![],
        checker: String::from("@MASTER:ACCOUNT"),
    };

    let db_info = (
        state.client_addr,
        state.config.0.connection.connection_password.clone(),
    );

    if username == "master" {
        if password != state.config.1.master_password {
            return Api::error(StatusCode::BAD_REQUEST, "account password error.");
        }
        account_info.usable = true;
    } else {
        // 通过数据库读取相关用户账号信息
        let accounts = db::accounts(db_info).await;

        if !accounts.contains_key(&username) {
            return Api::error(StatusCode::BAD_REQUEST, "unknown account.");
        }

        let info = accounts.get(&username).unwrap().clone();
        if !info.usable {
            return Api::error(StatusCode::BAD_REQUEST, "account disable.");
        }

        if info.password != password {
            return Api::error(StatusCode::BAD_REQUEST, "account password error.");
        }

        account_info = info;
    }

    let jwt = secret::Secret {
        token: state.config.1.token.clone(),
    };

    let v = match jwt.apply(account_info.clone(), 60 * 60 * 12) {
        Ok(v) => v,
        Err(_) => {
            // 抛出生成器异常
            return Api::error(StatusCode::INTERNAL_SERVER_ERROR, "jwt apply error.");
        }
    };

    Api::json(
        StatusCode::OK,
        json!({
            "type": "JsonWebToken",
            "token": v,
            "level": username,
            "usa_db": account_info.usa_database.unwrap_or_default(),
            "cls_command": account_info.cls_command,
        }),
    )
}

// 数据库 Ping 检测（也可作为 JWT Validation 使用）
pub async fn ping(
    extract::TypedHeader(auth): extract::TypedHeader<
        headers::Authorization<headers::authorization::Bearer>,
    >,
    state: extract::Extension<Arc<ShareState>>,
) -> Api {
    let token = String::from(auth.0.token());

    let jwt = secret::Secret {
        token: state.config.1.token.clone(),
    };

    let _ = match jwt.validation(token) {
        Ok(v) => v,
        Err(e) => {
            return Api::error(
                StatusCode::UNAUTHORIZED,
                &format!("jwt check failed [{}].", e),
            );
        }
    };

    // 尝试连接 Dorea 服务器
    let client = DoreaClient::connect(
        state.client_addr,
        &state.config.0.connection.connection_password,
    )
    .await;

    match client {
        Ok(_) => Api::ok(),
        Err(e) => Api::error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// key: 数据键信息
/// value: 数据内容
/// expire: 过期时间
/// query: 直接运行内容
#[derive(Deserialize)]
pub struct ControllerForm {
    key: Option<String>,
    value: Option<String>,
    expire: Option<usize>,
    query: Option<String>,
    style: Option<String>,
}

// 接口主控入口
pub async fn controller(
    extract::Path((group, operation)): extract::Path<(String, String)>,
    form: Option<extract::Form<ControllerForm>>,
    state: extract::Extension<Arc<ShareState>>,
    extract::TypedHeader(auth): extract::TypedHeader<
        headers::Authorization<headers::authorization::Bearer>,
    >,
) -> Api {
    let token = String::from(auth.0.token());

    let jwt = secret::Secret {
        token: state.config.1.token.clone(),
    };

    let v = match jwt.validation(token) {
        Ok(v) => v,
        Err(e) => {
            return Api::error(
                StatusCode::UNAUTHORIZED,
                &format!("jwt check failed [{}].", e),
            );
        }
    };

    if &group[0..1] != "@" {
        return Api::error(StatusCode::BAD_REQUEST, "group name must start with @.");
    }

    let group = group[1..].to_string();

    let accinfo = v.claims.account.clone();
    let usals = accinfo.usa_database.clone();
    if usals.is_some() && !usals.unwrap().contains(&group) {
        return Api::error(
            StatusCode::UNAUTHORIZED,
            "token do not have access to this database.",
        );
    }

    // 尝试连接 Dorea 服务器
    let client = DoreaClient::connect(
        state.client_addr,
        &state.config.0.connection.connection_password,
    )
    .await;

    let mut client = match client {
        Ok(c) => c,
        Err(e) => {
            return Api::error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    };

    let _ = match client.select(&group).await {
        Ok(_) => {}
        Err(_) => {
            return Api::error(StatusCode::INTERNAL_SERVER_ERROR, "Client execute failed.");
        }
    };

    let operation = operation.to_lowercase();

    if &operation == "info" || &operation == "information" {
        let keys = client.info(InfoType::KeyList).await.unwrap();
        let keys = serde_json::from_str::<Vec<String>>(&keys).unwrap_or_default();

        return Api::json(
            StatusCode::OK,
            json!({
                "group_name": &group,
                "key_list": keys,
                "key_number": keys.len(),
            }),
        );
    } else if &operation == "get" {
        let form = match form {
            Some(v) => v,
            None => {
                return Api::error(StatusCode::BAD_REQUEST, "form data not found.");
            }
        };

        if form.key.is_none() {
            return Api::lose_param("key");
        }

        let style: &str;
        if form.style.is_none() {
            style = "doson";
        } else {
            let temp_style = form.style.clone().unwrap();
            if temp_style.to_lowercase() == "json" {
                style = "json";
            } else {
                style = "doson";
            }
        }

        let _ = client.execute(&format!("value style {}", style)).await;

        let key = form.key.clone().unwrap();

        let value = client.get(&key).await;

        return match value {
            Some(v) => Api::json(
                StatusCode::OK,
                json!({
                    "key": key,
                    "value": v.to_string(),
                    "type": v.datatype()
                }),
            ),
            None => Api::not_found(&key),
        };
    } else if &operation == "set" {
        let form = match form {
            Some(v) => v,
            None => {
                return Api::error(StatusCode::BAD_REQUEST, "form data not found.");
            }
        };

        if form.key.is_none() {
            return Api::lose_param("key");
        }

        let key = form.key.clone().unwrap();

        if form.value.is_none() {
            return Api::lose_param("value");
        }

        let value = form.value.clone().unwrap();

        let value = DataValue::from(&value);
        if value == DataValue::None {
            return Api::error(StatusCode::BAD_REQUEST, "value parse error.");
        }

        let mut expire = 0;
        if let Some(v) = form.expire {
            expire = v;
        }

        let result = client.setex(&key, value, expire).await;

        return match result {
            Ok(_) => Api::ok(),
            Err(e) => Api::error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        };
    } else if &operation == "delete" {
        let form = match form {
            Some(v) => v,
            None => {
                return Api::error(StatusCode::BAD_REQUEST, "form data not found.");
            }
        };

        if form.key.is_none() {
            return Api::lose_param("key");
        }

        let key = form.key.clone().unwrap();

        return match client.delete(&key).await {
            Ok(_) => Api::ok(),
            Err(e) => Api::error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        };
    } else if &operation == "clean" {
        // 清空所有数据
        return match client.clean().await {
            Ok(_) => Api::ok(),
            Err(e) => Api::error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        };
    } else if &operation == "execute" {
        // 尝试直接运行 execute raw 数据。

        let form = match form {
            Some(v) => v,
            None => {
                return Api::error(StatusCode::BAD_REQUEST, "form data not found.");
            }
        };

        let style: &str;
        if form.style.is_none() {
            style = "doson";
        } else {
            let temp_style = form.style.clone().unwrap();
            if temp_style.to_lowercase() == "json" {
                style = "json";
            } else {
                style = "doson";
            }
        }

        let _ = client.execute(&format!("value style {}", style)).await;

        if form.query.is_none() {
            return Api::lose_param("query");
        }

        let query = form.query.clone().unwrap();

        // we will check the `cls_command` list
        // if the command is in the cls_command list, we can't execute it.
        let split_cmd = query.split(' ').collect::<Vec<&str>>();
        for patt in accinfo.cls_command.iter() {
            let mut matched = true;
            let patt_sec = patt.split('@');
            if patt_sec.clone().count() > split_cmd.len() {
                matched = false;
            }
            for (index, sec) in patt_sec.enumerate() {
                if !matched {
                    break;
                }
                if sec != split_cmd[index] {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Api::error(StatusCode::UNAUTHORIZED, "account can't use this command.");
            }
        }

        // return Api::error(StatusCode::UNAUTHORIZED, "account can't use this command.");

        let v = match client.execute(&query).await {
            Ok(v) => v,
            Err(e) => {
                return Api::error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
            }
        };

        if v.0 == NetPacketState::OK {
            return Api::json(
                StatusCode::OK,
                json!({
                    "reply": String::from_utf8_lossy(&v.1[..]).to_string()
                }),
            );
        } else {
            return Api::error(
                StatusCode::BAD_REQUEST,
                &String::from_utf8_lossy(&v.1[..]).to_string(),
            );
        }
    } else if &operation == "d2j" {
        let form = match form {
            Some(v) => v,
            None => {
                return Api::error(StatusCode::BAD_REQUEST, "form data not found.");
            }
        };

        if form.query.is_none() {
            return Api::lose_param("query");
        }

        let query = form.query.clone().unwrap();

        let dv = crate::value::DataValue::from(&query);

        return Api::json(
            StatusCode::OK,
            json!({
                "reply": dv.to_json()
            }),
        );
    } else if &operation == "j2d" {
        let form = match form {
            Some(v) => v,
            None => {
                return Api::error(StatusCode::BAD_REQUEST, "form data not found.");
            }
        };

        if form.query.is_none() {
            return Api::lose_param("query");
        }

        let query = form.query.clone().unwrap();

        let dv = crate::value::DataValue::from_json(&query);

        return Api::json(
            StatusCode::OK,
            json!({
                "reply": dv.to_string()
            }),
        );
    }

    Api::error(StatusCode::BAD_REQUEST, "operation not found.")
}

#[allow(clippy::single_match)]
pub async fn socket_handler(
    ws: WebSocketUpgrade,
    _user_agent: Option<TypedHeader<headers::UserAgent>>,
    state: extract::Extension<Arc<ShareState>>,
) -> impl axum::response::IntoResponse {
    let db_info = (
        state.client_addr,
        state.config.0.connection.connection_password.clone(),
    );

    let mut client = DoreaClient::connect(
        state.client_addr,
        &state.config.0.connection.connection_password,
    )
    .await
    .unwrap();

    ws.on_upgrade(move |mut socket| async move {
        let mut account_info = db::ServiceAccountInfo {
            usable: false,
            username: "ghost".to_string(),
            password: "ghost".to_string(),
            usa_database: None,
            cls_command: vec![],
            checker: String::from("@MASTER:ACCOUNT"),
        };

        loop {
            if let Some(Ok(message)) = socket.recv().await {
                match message {
                    Message::Text(content) => {
                        let commands = content.split(' ').collect::<Vec<&str>>();
                        let command_name: &str = commands.get(0).unwrap();

                        let usable_db = account_info.usa_database.clone();
                        let closed_command = account_info.cls_command.clone();

                        if account_info.usable {
                            match command_name {
                                "select" => {
                                    if commands.len() == 2 {
                                        let target = commands.get(1).unwrap().to_string();
                                        if usable_db.is_none()
                                            || usable_db.unwrap().contains(&target)
                                        {
                                            client.select(&target).await.unwrap();
                                        } else {
                                            socket
                                                .send(ws_error("Account permission denied"))
                                                .await
                                                .unwrap();
                                        }
                                    } else {
                                        socket
                                            .send(ws_error("Parameters number error"))
                                            .await
                                            .unwrap();
                                    }
                                }
                                command_name => {
                                    if !closed_command.contains(&command_name.to_string()) {
                                        if let Ok(v) = client.execute(&content).await {
                                            if let NetPacketState::OK = v.0 {
                                                socket
                                                    .send(ws_info(serde_json::Value::String(
                                                        String::from_utf8_lossy(&v.1[..])
                                                            .to_string(),
                                                    )))
                                                    .await
                                                    .unwrap();
                                            } else {
                                                socket
                                                    .send(ws_error(&String::from_utf8_lossy(
                                                        &v.1[..],
                                                    )))
                                                    .await
                                                    .unwrap();
                                            }
                                        } else {
                                            socket
                                                .send(ws_error("Client execute failed"))
                                                .await
                                                .unwrap();
                                        }
                                    } else {
                                        socket
                                            .send(ws_error("Account permission denied"))
                                            .await
                                            .unwrap();
                                    }
                                }
                            }
                        } else if command_name == "login" {
                            if commands.len() == 3 {
                                let username = commands.get(1).unwrap().to_string();
                                let password = commands.get(2).unwrap().to_string();

                                if username == "master" {
                                    if password != state.config.1.master_password {
                                        let _ =
                                            socket.send(ws_error("Account password error")).await;
                                    } else {
                                        account_info.usable = true;
                                    }
                                } else {
                                    // 通过数据库读取相关用户账号信息
                                    let accounts = db::accounts(db_info.clone()).await;

                                    if !accounts.contains_key(&username) {
                                        let _ = socket.send(ws_error("Unknown account")).await;
                                    } else {
                                        let info = accounts.get(&username).unwrap().clone();
                                        if !info.usable {
                                            let _ = socket.send(ws_error("Account disable")).await;
                                        } else if info.password != password {
                                            let _ = socket
                                                .send(ws_error("Account password error"))
                                                .await;
                                        } else {
                                            account_info = info;
                                        }
                                    }
                                }

                                // 这里说明登录是成功的
                                if account_info.usable {
                                    let _ = socket
                                        .send(ws_info(serde_json::Value::String(
                                            serde_json::to_string(&account_info)
                                                .unwrap_or_default(),
                                        )))
                                        .await;
                                }
                            } else {
                                let _ = socket.send(ws_error("Missing command parameters")).await;
                            }
                        } else {
                            let _ = socket.send(ws_error("Authentication failed")).await;
                        }
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => { /* empty code block */ }
                }
            }
        }
    })
}

fn ws_error(info: &str) -> Message {
    let res = json!({
        "alpha": "ERR",
        "data": {},
        "message": info,
    })
    .to_string();

    Message::Text(res)
}

fn ws_info(info: serde_json::Value) -> Message {
    let res = json!({
        "alpha": "OK",
        "data": info,
        "message": "",
    })
    .to_string();

    Message::Text(res)
}

// pub type Api = (StatusCode ,Json<serde_json::Value>);

pub struct Api {
    status: StatusCode,
    data: Json<serde_json::Value>,
}

impl axum::response::IntoResponse for Api {
    fn into_response(self) -> Response {
        let mut res = self.data.into_response();
        *res.status_mut() = self.status;
        res
    }
}

impl Api {
    pub fn error(code: StatusCode, message: &str) -> Api {
        Api {
            status: code,
            data: Json(json!({
                "alpha": "ERR",
                "data": {},
                "message": message.to_string(),
                "resptime": chrono::Local::now().timestamp()
            })),
        }
    }

    pub fn ok() -> Api {
        Api::json(StatusCode::OK, json!({}))
    }

    pub fn json(code: StatusCode, value: serde_json::Value) -> Api {
        Api {
            status: code,
            data: Json(json!({
                "alpha": "OK",
                "data": value,
                "message": "",
                "resptime": chrono::Local::now().timestamp()
            })),
        }
    }

    pub fn lose_param(name: &str) -> Api {
        Api::error(
            StatusCode::BAD_REQUEST,
            &format!("required parameter `{}` does not exist.", name),
        )
    }

    pub fn not_found(name: &str) -> Api {
        Api::error(
            StatusCode::NOT_FOUND,
            &format!("data `{}` not found.", name),
        )
    }
}
