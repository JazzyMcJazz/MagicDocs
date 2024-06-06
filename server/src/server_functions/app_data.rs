use super::models::AppData;
use leptos::{server, ServerFnError};

#[server]
pub async fn get_app_data() -> Result<AppData, ServerFnError> {
    use super::models::UserData;
    use crate::utils::claims::Claims;
    use axum::Extension;
    use leptos_axum::extract;

    let claims: Extension<Claims> = extract().await?;
    let user = UserData {
        id: claims.sub(),
        email: claims.email(),
        name: claims.name(),
        preferred_username: claims.username(),
        given_name: claims.given_name(),
        family_name: claims.family_name(),
        roles: claims.roles(),
        is_admin: claims.is_admin(),
        is_super_admin: claims.is_super_admin(),
    };

    Ok(AppData { user })
}
