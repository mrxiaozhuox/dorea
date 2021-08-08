//!
//! 关于 Rest-Service API 接口结构声明
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
use axum::http::StatusCode;

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
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(
            json!({
                "alpha": "ERR",
                "data": {},
                "message" : "password field not found."
            })
        ));
    }

    let password = v.get("password").unwrap();

    if password.is_file() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(
            json!({
                "alpha": "ERR",
                "data": {},
                "message" : "password type error."
            })
        ));
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
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(
            json!({
                "alpha": "ERR",
                "data": {},
                "message": "account info not found."
            })
        ));
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
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(
                json!({
                    "alpha": "ERR",
                    "data": {},
                    "message" :e.to_string()
                })
            ));
        }
    };

    (
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
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "alpha": "ERR",
                    "data": {},
                    "message": e.to_string()
                }))
            );
        }
    };

    (StatusCode::OK, Json(json!({
        "alpha": "OK",
        "data": {},
        "message": ""
    })))
}

pub type Api = (StatusCode ,Json<serde_json::Value>);