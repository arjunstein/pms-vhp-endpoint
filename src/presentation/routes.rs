use crate::presentation::handlers::pms_handler;
use salvo::oapi::OpenApi;
use salvo::prelude::*;

pub fn router() -> Router {
    let api_router = Router::with_path("/vhp").get(pms_handler);

    let doc = OpenApi::default().merge_router(&api_router);

    Router::new()
        .push(api_router)
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/api-doc/openapi.json").into_router("/documentation"))
}
