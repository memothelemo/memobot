// TODO: clean messy actix code
use actix_web::{http::header, web, HttpResponse};
use constant_time_eq::constant_time_eq;
use futures::future::BoxFuture;

pub mod sanctuary;

#[derive(Debug)]
pub enum ApiAuthorizationError {
    InvalidToken,
    NoParadiseConfig,
}

impl std::fmt::Display for ApiAuthorizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("failed to authorize user")
    }
}

impl actix_web::ResponseError for ApiAuthorizationError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            ApiAuthorizationError::InvalidToken => {
                HttpResponse::Unauthorized().body("401 Unauthorized")
            }
            ApiAuthorizationError::NoParadiseConfig => {
                HttpResponse::NotFound().body("404 Not Found")
            }
        }
    }
}

pub struct ApiAuthorization(crate::Service);

impl ApiAuthorization {
    pub fn service(&self) -> crate::Service {
        self.0.clone()
    }
}

impl actix_web::FromRequest for ApiAuthorization {
    type Error = ApiAuthorizationError;
    type Future = BoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let service = req.app_data::<web::Data<Option<crate::Service>>>();
        let service = service.and_then(|v| v.as_ref().as_ref().clone());
        let Some(service) = service else {
            tracing::warn!(
                "user tried to access resource with Paradise server configuration is disabled"
            );
            return Box::pin(futures::future::err(
                ApiAuthorizationError::NoParadiseConfig,
            ));
        };

        let token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|v| v.to_string())
            .unwrap_or_default();

        // Performing "timing-safe equal"
        let actual_token = service.config().token().as_bytes();
        if !constant_time_eq(actual_token, token.as_bytes()) {
            tracing::warn!("user tried to access resource with invalid token");
            return Box::pin(futures::future::err(ApiAuthorizationError::InvalidToken));
        }

        Box::pin(futures::future::ok(ApiAuthorization(service.clone())))
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("sanctuary", web::post().to(sanctuary::alert_status));
}
