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
use axum::response::{Html, Json};
use serde_json::json;
use std::sync::Arc;

use crate::service::ShareState;
use crate::service::secret;
use axum::http::{StatusCode, Response};
use crate::client::DoreaClient;

pub async fn index() -> Html<&'static str> {
    Html("<h1>Hello World</h1>")
}

// 授权系统（JWT 发放）控制函数
pub async fn auth(
    multipart: extract::Multipart,
    state: extract::Extension<Arc<ShareState>>,
) -> Api {

    let v = crate::service::tools::multipart(multipart).await;

    if ! v.contains_key("password") {
        return Api::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "password field not found."
        );
    }

    let password = v.get("password").unwrap();

    if password.is_file() {
        return Api::error(StatusCode::INTERNAL_SERVER_ERROR, "password type error.");
    }

    let v = &state.config.1.account;
    let mut account: String = String::new();
    for i in v {
        if i.1.to_string() == password.text().unwrap_or(String::new()) {
            account = i.0.to_string();
            break;
        }
    }

    if account == String::new() {
        return Api::error(StatusCode::INTERNAL_SERVER_ERROR, "account info not found.");
    }

    let jwt = secret::Secret {
        token: state.config.1.foundation.token.clone()
    };

    let v = match jwt.apply(
        account.clone(),
        60 * 60 * 12
    ) {
        Ok(v) => v,
        Err(e) => {
            // 抛出生成器异常
            return Api::error(StatusCode::INTERNAL_SERVER_ERROR, "jwt apply error.");
        }
    };

    Api::json(
        StatusCode::OK,
        Json(json!({
            "alpha": "OK",
            "data": {
                "type": "JsonWebToken",
                "token": v,
                "level": account
            },
            "message": ""
        }))
    )
}

// 数据库 Ping 检测（也可作为 JWT Validation 使用）
pub async fn ping(
    extract::TypedHeader(auth): extract::TypedHeader<
        headers::Authorization<headers::authorization::Bearer>
    >,
    state: extract::Extension<Arc<ShareState>>,
) -> Api {

    let token = String::from(auth.0.token());

    let jwt = secret::Secret {
        token: state.config.1.foundation.token.clone()
    };

    let _v = match jwt.validation(token) {
        Ok(v) => v,
        Err(e) => {
            return Api::error(StatusCode::UNAUTHORIZED, &e.to_string());
        }
    };

    // 尝试连接 Dorea 服务器
    let client = DoreaClient::connect(
        state.client_addr,
        &state.config.0.connection.connection_password
    ).await;

    return match client {
        Ok(_) => {
            Api::reply(StatusCode::OK, "PONG")
        },
        Err(e) => {
            Api::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                &e.to_string()
            )
        }
    }
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
                "message": message.to_string()
            }))
        }
    }

    pub fn reply(code: StatusCode, reply: &str) -> Api {
        Api {
            status: code,
            data: Json(json!({
                "alpha": "ERR",
                "data": reply.to_string(),
                "message": ""
            }))
        }
    }

    pub fn json(code: StatusCode, json: Json<serde_json::Value>) -> Api {
        Api {
            status: code,
            data: json,
        }
    }
}