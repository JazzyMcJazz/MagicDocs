use axum::{extract::State, response::Response, Extension};

use crate::{server::AppState, utils::traits::TryRender};

// GET /
pub async fn index(
    State(state): State<AppState>,
    Extension(context): Extension<tera::Context>,
) -> Response {
    state.tera.try_render("index.html", &context)
}
