use axum::Router;

pub fn build_router() -> Router {
    Router::new().route("get-feedback", get(get_feedback::handle))
}
