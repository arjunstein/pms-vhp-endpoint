use crate::presentation::handlers::pms_handler;
use salvo::prelude::*;

pub fn router() -> Router {
    Router::new().push(Router::with_path("/vhp").get(pms_handler))
}
