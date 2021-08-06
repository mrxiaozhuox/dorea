use std::sync::Arc;
use std::path::PathBuf;
use axum::{route, AddExtensionLayer};
use axum::prelude::*;

pub mod routes;
pub mod secret;

pub struct ShareState {
    pub(crate) config: (crate::configure::DoreaFileConfig, crate::configure::RestConfig)
}

pub async fn make_service(
    hostname: &'static str,
    document_path: &PathBuf
) -> crate::Result<()> {

    // 读取 rest-service path
    let rest_config = crate::configure::load_rest_config(&document_path)?;

    if ! rest_config.foundation.switch {
        return Ok(());
    }

    // 全局共享状态数据
    let share_state = Arc::new(
        ShareState {
            config: (
                crate::configure::load_config(&document_path).unwrap(),
                rest_config.clone(),
            )
        }
    );

    let rest_port = rest_config.foundation.port;
    tokio::task::spawn(async move {

        let app = route(
            "/", get(routes::index)
        )
            .route(
                "/auth", get(routes::auth)
            )
            .layer(AddExtensionLayer::new(share_state));

        let addr = format!(
            "{}:{}",
            hostname,
            rest_port
        );

        log::info!("⍹ >>> Rest-Service Running at: http://{}/", addr);

        hyper::Server::bind(&addr.parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(())
}