use actix_web::{web, HttpResponse};
use serde::Deserialize;

use super::ApiAuthorization;

#[derive(Debug, Clone, Deserialize)]
pub struct AlertSanctuaryStatusParams {
    pub online: bool,
}

// `memobot` crate will handle rate limiting on this
#[tracing::instrument(skip_all, fields(
    service = ?authorization.0,
    params.online = %params.online
))]
pub async fn alert_status(
    params: web::Query<AlertSanctuaryStatusParams>,
    authorization: ApiAuthorization,
) -> HttpResponse {
    // I guess, we can just let the bot know about it
    let service = authorization.service();
    let kernel = service.kernel().clone();
    let service = service.clone();

    kernel.spawn(async move {
        if let Err(error) = crate::bot::sanctuary::alert_everyone(&service, params.online).await {
            tracing::error!(?error, "Failed to alert everyone in Paradise guild");
        }
    });

    HttpResponse::Ok().body("Ok!")
}
