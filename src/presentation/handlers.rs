use crate::application::{
    dtos::{PmsQueryParams, PmsResponse},
    services::BookingService,
};
use crate::infrastructure::{database::db_pool, repositories::MySqlBookingRepository};
use salvo::prelude::*;
use std::sync::Arc;

#[endpoint(
    parameters(PmsQueryParams),
    responses(
        (status_code = 200, body = PmsResponse, description = "Success",
            example = json!({
                "status": "success",
                "message": "room {} successfully checkin|checkout|update",
            })
        ),
        (status_code = 400, body = PmsResponse, description = "Bad Request",
            example = json!({
                "status": "error",
                "message": "invalid query params"
            })
        ),
        (status_code = 500, body = PmsResponse, description = "Internal Server Error",
            example = json!({
                "status": "error",
                "message": "internal server error"
            })
        )
    )
)]
pub async fn pms_handler(req: &mut Request, res: &mut Response) {
    // 1️⃣ Get connection pool from depot (shared state)
    let pool = db_pool();
    let repo = Arc::new(MySqlBookingRepository { pool: pool.clone() });
    let service = BookingService::new(repo);

    // 2️⃣ Parse query params to struct
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

    let mode = query.mode.clone();

    // 3️⃣ Run service logic
    match service.process(query).await {
        Ok(resp) => {
            res.status_code(StatusCode::OK);
            res.render(Json(resp));
        }
        Err(e) => {
            let err_msg = e.to_string();
            tracing::error!("{} failed: {:?}", mode, err_msg);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(PmsResponse {
                status: "error".into(),
                message: format!("{} failed: {}", mode, err_msg),
            }));
        }
    }
}
