use axum::prelude::*;
use axum::response::{Html, Json};
use serde_json::json;
use std::sync::Arc;

use crate::service::ShareState;
use crate::service::secret;

pub async fn index() -> Html<&'static str> {
    Html("<h1>Hello World</h1>")
}

// 授权系统（JWT 发放）控制函数
pub async fn auth(
    mut multipart: extract::Multipart,
    state: extract::Extension<Arc<ShareState>>,
) -> Json<serde_json::Value> {

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
    }

    let jwt = secret::Secret {
        token: state.config.1.foundation.token.clone()
    };

    let v = jwt.apply(
        "master".to_string(),
        10
    );

    Json(json!({
        // "password": payload.password,
        "token": v.unwrap()
    }))
}