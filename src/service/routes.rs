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

use axum::prelude::*;
use axum::response::Json;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::client::{DoreaClient, InfoType};
use crate::network::NetPacketState;
use crate::service::secret;
use crate::service::ShareState;
use crate::value::DataValue;
use axum::http::{Response, StatusCode};

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

    if username == *"master" {
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
                &format!("jwt check failed [{}].", e.to_string()),
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
                &format!("jwt check failed [{}].", e.to_string()),
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

// pub type Api = (StatusCode ,Json<serde_json::Value>);

pub struct Api {
    status: StatusCode,
    data: Json<serde_json::Value>,
}

impl axum::response::IntoResponse for Api {
    fn into_response(self) -> Response<Body> {
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
