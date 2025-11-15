use crate::application::{
    dtos::{PmsQueryParams, PmsResponse},
    errors::ErrorResponse,
    services::BookingService,
};
use crate::infrastructure::{database::db_pool, repositories::MySqlBookingRepository};
use salvo::prelude::*;
use std::sync::Arc;

#[endpoint(
    parameters(PmsQueryParams),
    responses(
        (status_code = 200, body = PmsResponse, description = "success", example = json!({
            "status": "success",
            "message": "room {} successfully checkin|checkout|update",
        })),
        (status_code = 400, body = PmsResponse, description = "bad request", example = json!({
            "status": "error",
            "message": "bad request",
        })),
        (status_code = 404, body = PmsResponse, description = "not found", example = json!({
            "status": "error",
            "message": "not found",
        })),
        (status_code = 500, body = PmsResponse, description = "internal server error", example = json!({
            "status": "error",
            "message": "internal server error",
        })),
    )
)]
pub async fn pms_handler(req: &mut Request, res: &mut Response) {
    let pool = db_pool();
    let repo = Arc::new(MySqlBookingRepository { pool: pool.clone() });
    let service = BookingService::new(repo);

    let query = match req.parse_queries::<PmsQueryParams>() {
        Ok(q) => q,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(PmsResponse {
                status: "error".into(),
                message: "invalid query params".into(),
            }));
            return;
        }
    };

    match service.process(query).await {
        Ok(resp) => {
            res.status_code(StatusCode::OK);
            res.render(Json(resp));
        }

        Err(ErrorResponse::Validation(msg)) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Json(PmsResponse {
                status: "error".into(),
                message: msg,
            }));
        }

        Err(ErrorResponse::NotFound(msg)) => {
            res.status_code(StatusCode::NOT_FOUND);
            res.render(Json(PmsResponse {
                status: "error".into(),
                message: msg,
            }));
        }

        Err(ErrorResponse::InternalServerErr(msg)) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(PmsResponse {
                status: "error".into(),
                message: msg,
            }));
        }
    }
}
