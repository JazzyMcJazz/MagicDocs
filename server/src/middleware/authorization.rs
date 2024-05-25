use axum::{
    extract::{Path, Request, State},
    middleware::Next,
    response::Response,
};
use entity::sea_orm_active_enums::PermissionEnum;
use http::Method;
use serde::Serialize;

use crate::{
    database::Repo, models::Slugs, responses::HttpResponse, server::AppState,
    utils::extractor::Extractor,
};

#[derive(Debug, Clone, Serialize)]
pub struct Permissions {
    read: bool,
    write: bool,
    delete: bool,
}

impl Permissions {
    pub fn _read(&self) -> bool {
        self.read
    }

    pub fn write(&self) -> bool {
        self.write
    }

    pub fn _delete(&self) -> bool {
        self.delete
    }
}

pub async fn authorization(
    State((admin, state)): State<(bool, AppState)>,
    Path(path): Path<Slugs>,
    mut req: Request,
    next: Next,
) -> Response {
    let user = match Extractor::claims(&req) {
        Ok(claims) => claims,
        Err(e) => {
            tracing::error!("Error extracting claims: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let mut permissions = Permissions {
        read: false,
        write: false,
        delete: false,
    };

    let admin_authorized = user.is_super_admin() || admin && user.is_admin();
    if admin_authorized {
        permissions = Permissions {
            read: true,
            write: true,
            delete: true,
        };

        let mut context = Extractor::context(&req);
        context.insert("permissions", &permissions);
        req.extensions_mut().insert(context);
    } else if let Some(project_id) = path.project_id() {
        let permission_type = if req.uri().path().ends_with("/chat") {
            // Chat is a special case, it only requires read permission
            PermissionEnum::Read
        } else {
            match *req.method() {
                Method::GET => PermissionEnum::Read,
                Method::POST => PermissionEnum::Create,
                Method::PUT => PermissionEnum::Create,
                Method::PATCH => PermissionEnum::Create,
                Method::DELETE => PermissionEnum::Delete,
                _ => return HttpResponse::MethodNotAllowed().finish(),
            }
        };

        let role_permissions = match state
            .conn
            .role_permissions()
            .find_by_project_id_in_roles(&user.roles(), &project_id)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("Error finding role permissions: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let user_permissions = match state
            .conn
            .user_permissions()
            .find_by_user_and_project_id(&user.sub(), &project_id)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                tracing::error!("Error finding user permissions: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

        let project_permissions = role_permissions
            .iter()
            .map(|role_permission| role_permission.r#type.to_owned())
            .chain(
                user_permissions
                    .iter()
                    .map(|user_permission| user_permission.r#type.to_owned()),
            )
            .collect::<Vec<_>>();

        if !project_permissions.contains(&permission_type) {
            return HttpResponse::Forbidden().finish();
        };

        permissions = Permissions {
            read: project_permissions.contains(&PermissionEnum::Read),
            write: project_permissions.contains(&PermissionEnum::Create),
            delete: project_permissions.contains(&PermissionEnum::Delete),
        };

        let mut context = Extractor::context(&req);
        context.insert("permissions", &permissions);
        req.extensions_mut().insert(context);
    }

    req.extensions_mut().insert(permissions);

    next.run(req).await
}
