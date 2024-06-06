use std::collections::HashMap;

use axum::{
    extract::{Path, Request, State},
    response::{IntoResponse, Response},
    Form,
};
use entity::sea_orm_active_enums::PermissionEnum;
use http::StatusCode;

use crate::{
    database::Repo,
    keycloak::{Keycloak, KeycloakRole},
    models::{CreateRole, Slugs},
    responses::HttpResponse,
    server::AppState,
    utils::{
        extractor::Extractor,
        traits::{Htmx, TryRender},
    },
};

pub async fn dashboard(data: State<AppState>, req: Request) -> Response {
    let tera = &data.tera;
    let context = Extractor::context(&req);

    let Ok(html) = tera.render("admin/dashboard.html", &context) else {
        return HttpResponse::InternalServerError()
            .body("Template error")
            .finish();
    };

    HttpResponse::Ok().body(html)
}

pub async fn users(data: State<AppState>, req: Request) -> Response {
    let Ok(tokens) = Extractor::tokens(&req) else {
        return HttpResponse::Unauthorized().finish();
    };

    let users = Keycloak::get_users(&tokens, None).await.unwrap_or_default();

    let mut context = Extractor::context(&req);
    context.insert("users", &users);

    let Ok(html) = data.tera.render("admin/users.html", &context) else {
        return HttpResponse::InternalServerError()
            .body("Template error")
            .finish();
    };

    HttpResponse::Ok().body(html)
}

pub async fn user_details(
    data: State<AppState>,
    Path(path): Path<Slugs>,
    req: Request,
) -> Response {
    let Ok(tokens) = Extractor::tokens(&req) else {
        return HttpResponse::Unauthorized().finish();
    };

    let user_id = path.user_id().unwrap_or_default();
    let Ok(user) = Keycloak::get_user(&tokens, &user_id).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(user_roles) = Keycloak::get_user_roles(&tokens, &user_id).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let Ok(available_roles) = Keycloak::get_user_available_roles(&tokens, &user_id).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let db = &data.conn;
    let projects = match db.projects().all_with_user_permissions(&user_id).await {
        Ok(projects) => projects,
        Err(e) => {
            tracing::error!("Error fetching projects: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut context = Extractor::context(&req);
    context.insert("kc_user", &user);
    context.insert("assigned_roles", &user_roles);
    context.insert("available_roles", &available_roles);
    context.insert("user_projects", &projects);

    data.tera.try_render("admin/user.html", &context)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserUpdate {
    pub permissions: HashMap<i32, UpdateRole>,
    pub roles_to_assign: Vec<KeycloakRole>,
    pub roles_to_revoke: Vec<KeycloakRole>,
}

pub async fn update_user(
    data: State<AppState>,
    Path(path): Path<Slugs>,
    req: Request,
) -> impl IntoResponse {
    let Ok(tokens) = Extractor::tokens(&req) else {
        return HttpResponse::Unauthorized().finish();
    };

    let user_id = path.user_id().unwrap_or_default();

    let user_update = match Extractor::json::<UserUpdate>(req).await {
        Ok(user_update) => user_update,
        Err(e) => {
            tracing::error!("Error parsing form data: {:?}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    let db = &data.conn;
    let mut create_permissions = Vec::new();
    let mut delete_permissions = Vec::new();

    for (project_id, role) in user_update.permissions.iter() {
        if let Some(val) = role.read {
            if val {
                create_permissions.push((*project_id, PermissionEnum::Read));
            } else {
                delete_permissions.push((*project_id, PermissionEnum::Read));
            }
        }
        if let Some(val) = role.write {
            if val {
                create_permissions.push((*project_id, PermissionEnum::Create));
            } else {
                delete_permissions.push((*project_id, PermissionEnum::Create));
            }
        }
        if let Some(val) = role.delete {
            if val {
                create_permissions.push((*project_id, PermissionEnum::Delete));
            } else {
                delete_permissions.push((*project_id, PermissionEnum::Delete));
            }
        }
    }

    if !create_permissions.is_empty() {
        let _ = db
            .user_permissions()
            .create_many_for_user(&user_id, create_permissions)
            .await;
    }

    if !delete_permissions.is_empty() {
        let _ = db
            .user_permissions()
            .delete_many_for_user(&user_id, delete_permissions)
            .await;
    }

    match Keycloak::revoke_user_roles(&tokens, &user_id, &user_update.roles_to_revoke).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error removing roles: {:?}", e);
        }
    }

    match Keycloak::grant_user_roles(&tokens, &user_id, &user_update.roles_to_assign).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error assigning roles: {:?}", e);
        }
    }

    HttpResponse::Ok().body("Update user")
}

pub async fn roles(data: State<AppState>, req: Request) -> impl IntoResponse {
    let Ok(tokens) = Extractor::tokens(&req) else {
        return HttpResponse::Unauthorized().finish();
    };

    let Ok(roles) = Keycloak::get_client_roles(&tokens, None).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let roles = roles
        .iter()
        .filter(|role| role.name().ne("admin"))
        .collect::<Vec<_>>();

    let mut context = Extractor::context(&req);
    context.insert("roles", &roles);

    data.tera.try_render("admin/roles.html", &context)
}

pub async fn create_role(req: Request) -> impl IntoResponse {
    let Ok(tokens) = Extractor::tokens(&req) else {
        return HttpResponse::Unauthorized().finish();
    };

    let (status, header) = req.headers().redirect_status_and_header();

    let Ok(Form(role)) = Extractor::form_data::<CreateRole>(req).await else {
        return HttpResponse::BadRequest().finish();
    };

    if role.name.is_empty() {
        return HttpResponse::BadRequest().finish();
    }

    match Keycloak::create_client_role(&tokens, &role).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Error creating role: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    match Keycloak::get_client_roles(&tokens, Some(&role.name)).await {
        Ok(roles) => {
            let Some(role) = roles.first() else {
                return HttpResponse::InternalServerError().finish();
            };

            HttpResponse::build(status)
                .insert_header((header, format!("/admin/roles/{}", role.id())))
                .finish()
        }
        Err(e) => {
            tracing::error!("Error fetching role: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn role_details(
    data: State<AppState>,
    Path(path): Path<Slugs>,
    req: Request,
) -> impl IntoResponse {
    let Ok(tokens) = Extractor::tokens(&req) else {
        return HttpResponse::Unauthorized().finish();
    };

    let role_name = path.role_name().unwrap_or_default();

    let Ok(role) = Keycloak::get_client_role_by_name(&tokens, &role_name).await else {
        return HttpResponse::InternalServerError().finish();
    };

    let db = &data.conn;
    let projects = match db.projects().all_with_role_permissions(role.name()).await {
        Ok(projects) => projects,
        Err(e) => {
            tracing::error!("Error fetching projects: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut context = Extractor::context(&req);
    context.insert("role", &role);
    context.insert("role_projects", &projects);

    data.tera.try_render("admin/role.html", &context)
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateRole {
    read: Option<bool>,
    write: Option<bool>,
    delete: Option<bool>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateRolePermissions {
    pub data: HashMap<i32, UpdateRole>,
}

pub async fn update_role_permissions(
    data: State<AppState>,
    Path(path): Path<Slugs>,
    req: Request,
) -> impl IntoResponse {
    let role_name = path.role_name().unwrap_or_default();

    let permissions = match Extractor::json::<UpdateRolePermissions>(req).await {
        Ok(permissions) => permissions,
        Err(e) => {
            tracing::error!("Error parsing form data: {:?}", e);
            return HttpResponse::BadRequest().finish();
        }
    };

    let mut create_permissions = Vec::new();
    let mut delete_permissions = Vec::new();
    for (project_id, role) in permissions.data.iter() {
        if let Some(val) = role.read {
            if val {
                create_permissions.push((*project_id, PermissionEnum::Read));
            } else {
                delete_permissions.push((*project_id, PermissionEnum::Read));
            }
        }
        if let Some(val) = role.write {
            if val {
                create_permissions.push((*project_id, PermissionEnum::Create));
            } else {
                delete_permissions.push((*project_id, PermissionEnum::Create));
            }
        }
        if let Some(val) = role.delete {
            if val {
                create_permissions.push((*project_id, PermissionEnum::Delete));
            } else {
                delete_permissions.push((*project_id, PermissionEnum::Delete));
            }
        }
    }

    let db = &data.conn;
    if !create_permissions.is_empty() {
        let _ = db
            .role_permissions()
            .create_many_for_role(&role_name, create_permissions)
            .await;
    }

    if !delete_permissions.is_empty() {
        let _ = db
            .role_permissions()
            .delete_many_for_role(&role_name, delete_permissions)
            .await;
    }

    HttpResponse::build(StatusCode::OK).finish()
}
