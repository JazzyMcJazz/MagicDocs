use leptos::{server, ServerFnError};

use super::models::{AppRole, AppUser, ProjectData, ProjectPermissions};

#[server]
pub async fn get_user_list() -> Result<Vec<AppUser>, ServerFnError> {
    use crate::keycloak::Keycloak;
    use crate::utils::claims::JwtTokens;
    use axum::Extension;
    use leptos_axum::extract;

    let tokens: Extension<JwtTokens> = extract().await?;

    let users = Keycloak::get_users(&tokens, None).await.unwrap_or_default();

    let users = users
        .iter()
        .map(|user| AppUser {
            created_timestamp: user.created_timestamp(),
            email: user.email().to_owned(),
            email_verified: user.email_verified(),
            enabled: user.enabled(),
            first_name: user.first_name().to_owned(),
            id: user.id().to_owned(),
            last_name: user.last_name().to_owned(),
            username: user.username().to_owned(),
        })
        .collect::<Vec<_>>();

    Ok(users)
}

#[server]
pub async fn get_user(
    user_id: String,
) -> Result<
    (
        AppUser,
        Vec<AppRole>,
        Vec<AppRole>,
        Vec<(ProjectData, ProjectPermissions)>,
    ),
    ServerFnError,
> {
    use crate::utils::claims::JwtTokens;
    use crate::{database::Repo, keycloak::Keycloak, server::AppState};
    use axum::Extension;
    use entity::sea_orm_active_enums::PermissionEnum;
    use leptos::use_context;
    use leptos_axum::extract;

    let tokens: Extension<JwtTokens> = extract().await?;

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let Ok(user) = Keycloak::get_user(&tokens, &user_id).await else {
        return Err(ServerFnError::ServerError("Error fetching user".to_owned()));
    };

    let Ok(assigned_roles) = Keycloak::get_user_roles(&tokens, &user_id).await else {
        return Err(ServerFnError::ServerError(
            "Error fetching user roles".to_owned(),
        ));
    };

    let Ok(available_roles) = Keycloak::get_user_available_roles(&tokens, &user_id).await else {
        return Err(ServerFnError::ServerError(
            "Error fetching available roles".to_owned(),
        ));
    };

    let projects = match state
        .conn
        .projects()
        .all_with_user_permissions(&user_id)
        .await
    {
        Ok(projects) => projects,
        Err(e) => {
            tracing::error!("Error fetching projects: {:?}", e);
            return Err(ServerFnError::ServerError(
                "Error fetching projects".to_owned(),
            ));
        }
    };

    // Map server models to client models
    let user = AppUser {
        created_timestamp: user.created_timestamp(),
        email: user.email().to_owned(),
        email_verified: user.email_verified(),
        enabled: user.enabled(),
        first_name: user.first_name().to_owned(),
        id: user.id().to_owned(),
        last_name: user.last_name().to_owned(),
        username: user.username().to_owned(),
    };

    let mut assigned_roles = assigned_roles
        .iter()
        .map(|role| AppRole {
            id: role.id().to_owned(),
            name: role.name().to_owned(),
            description: role.description(),
        })
        .collect::<Vec<_>>();
    assigned_roles.sort_by(|a, b| b.name.cmp(&a.name));

    let mut available_roles = available_roles
        .iter()
        .map(|role| AppRole {
            id: role.id().to_owned(),
            name: role.name().to_owned(),
            description: role.description(),
        })
        .collect::<Vec<_>>();
    available_roles.sort_by(|a, b| b.name.cmp(&a.name));

    let projects = projects
        .iter()
        .map(|(project, permissions)| {
            let project = ProjectData {
                id: project.id,
                name: project.name.to_owned(),
                description: project.description.to_owned(),
                version: -1,
                versions: Vec::new(),
                finalized: false,
            };

            let mut p = (false, false, false);
            for permission in permissions.iter() {
                match permission.r#type {
                    PermissionEnum::Read => p.0 = true,
                    PermissionEnum::Create | PermissionEnum::Update => p.1 = true,
                    PermissionEnum::Delete => p.2 = true,
                }
            }

            let permissions = ProjectPermissions {
                read: p.0,
                write: p.1,
                delete: p.2,
            };
            (project, permissions)
        })
        .collect::<Vec<_>>();

    Ok((user, assigned_roles, available_roles, projects))
}

#[server]
pub async fn update_user_privileges(
    user_id: String,
    roles_to_grant: String,
    roles_to_revoke: String,
    permissions_to_grant: String,
    permissions_to_revoke: String,
) -> Result<(), ServerFnError> {
    use super::models::Permission;
    use crate::{
        database::Repo,
        keycloak::{Keycloak, KeycloakRole},
        server::AppState,
        utils::claims::JwtTokens,
    };
    use axum::Extension;
    use entity::sea_orm_active_enums::PermissionEnum;
    use leptos::use_context;
    use leptos_axum::extract;

    let tokens: Extension<JwtTokens> = extract().await?;

    let Some(state) = use_context::<AppState>() else {
        tracing::error!("Failed to get app state");
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let db = state.conn;

    let Ok(roles_to_grant) = serde_json::from_str::<Vec<KeycloakRole>>(&roles_to_grant) else {
        tracing::error!("Error parsing roles to grant");
        return Err(ServerFnError::ServerError(
            "Error parsing roles to grant".to_owned(),
        ));
    };

    let Ok(roles_to_revoke) = serde_json::from_str::<Vec<KeycloakRole>>(&roles_to_revoke) else {
        tracing::error!("Error parsing roles to revoke");
        return Err(ServerFnError::ServerError(
            "Error parsing roles to revoke".to_owned(),
        ));
    };

    let permissions_to_grant = if let Ok(permissions) =
        serde_json::from_str::<Vec<(i32, Permission)>>(&permissions_to_grant)
    {
        permissions
            .iter()
            .map(|(id, permission)| {
                let r#type = match permission {
                    Permission::Read => PermissionEnum::Read,
                    Permission::Write => PermissionEnum::Create,
                    Permission::Delete => PermissionEnum::Delete,
                };
                (*id, r#type)
            })
            .collect::<Vec<_>>()
    } else {
        tracing::error!("Error parsing permissions to grant");
        return Err(ServerFnError::ServerError(
            "Error parsing permissions to grant".to_owned(),
        ));
    };

    let permissions_to_revoke = if let Ok(permissions) =
        serde_json::from_str::<Vec<(i32, Permission)>>(&permissions_to_revoke)
    {
        permissions
            .iter()
            .map(|(id, permission)| {
                let r#type = match permission {
                    Permission::Read => PermissionEnum::Read,
                    Permission::Write => PermissionEnum::Create,
                    Permission::Delete => PermissionEnum::Delete,
                };
                (*id, r#type)
            })
            .collect::<Vec<_>>()
    } else {
        tracing::error!("Error parsing permissions to revoke");
        return Err(ServerFnError::ServerError(
            "Error parsing permissions to revoke".to_owned(),
        ));
    };

    if !roles_to_grant.is_empty() {
        match Keycloak::grant_user_roles(&tokens, &user_id, &roles_to_grant).await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error assigning roles: {:?}", e);
            }
        }
    }

    if !roles_to_revoke.is_empty() {
        match Keycloak::revoke_user_roles(&tokens, &user_id, &roles_to_revoke).await {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Error removing roles: {:?}", e);
            }
        }
    }

    if !permissions_to_grant.is_empty() {
        let _ = db
            .user_permissions()
            .create_many_for_user(&user_id, permissions_to_grant)
            .await;
    }

    if !permissions_to_revoke.is_empty() {
        let _ = db
            .user_permissions()
            .delete_many_for_user(&user_id, permissions_to_revoke)
            .await;
    }

    Ok(())
}

#[server]
pub async fn get_role_list() -> Result<Vec<AppRole>, ServerFnError> {
    use crate::{keycloak::Keycloak, utils::claims::JwtTokens};
    use axum::Extension;
    use leptos_axum::extract;

    let tokens: Extension<JwtTokens> = extract().await?;

    let Ok(roles) = Keycloak::get_client_roles(&tokens, None).await else {
        return Err(ServerFnError::ServerError(
            "Error fetching roles".to_owned(),
        ));
    };

    let roles = roles
        .iter()
        .filter(|role| role.name().ne("admin"))
        .collect::<Vec<_>>();

    let roles = roles
        .iter()
        .map(|role| AppRole {
            id: role.id().to_owned(),
            name: role.name().to_owned(),
            description: role.description(),
        })
        .collect::<Vec<_>>();

    Ok(roles)
}

#[server(CreateRole)]
pub async fn create_role(name: String, description: Option<String>) -> Result<(), ServerFnError> {
    use crate::{keycloak::Keycloak, models::CreateRole, utils::claims::JwtTokens};
    use axum::Extension;
    use leptos_axum::extract;

    let tokens: Extension<JwtTokens> = extract().await?;

    let role = CreateRole {
        name,
        description: description.unwrap_or_default(),
    };

    let Ok(_) = Keycloak::create_client_role(&tokens, &role).await else {
        return Err(ServerFnError::ServerError("Error creating role".to_owned()));
    };

    Ok(())
}

#[server]
pub async fn get_role(
    name: String,
) -> Result<(AppRole, Vec<(ProjectData, ProjectPermissions)>), ServerFnError> {
    use crate::{database::Repo, keycloak::Keycloak, server::AppState, utils::claims::JwtTokens};
    use axum::Extension;
    use entity::sea_orm_active_enums::PermissionEnum;
    use leptos::use_context;
    use leptos_axum::extract;

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let tokens: Extension<JwtTokens> = extract().await?;

    let Ok(role) = Keycloak::get_client_role_by_name(&tokens, &name).await else {
        return Err(ServerFnError::ServerError("Error fetching role".to_owned()));
    };

    let db = &state.conn;
    let projects = match db.projects().all_with_role_permissions(role.name()).await {
        Ok(projects) => projects,
        Err(e) => {
            tracing::error!("Error fetching projects: {:?}", e);
            return Err(ServerFnError::ServerError(
                "Error fetching projects".to_owned(),
            ));
        }
    };

    let role = AppRole {
        id: role.id().to_owned(),
        name: role.name().to_owned(),
        description: role.description(),
    };

    let projects = projects
        .iter()
        .map(|(project, permissions)| {
            let project = ProjectData {
                id: project.id,
                name: project.name.to_owned(),
                description: project.description.to_owned(),
                version: -1,
                versions: Vec::new(),
                finalized: false,
            };

            let mut p = (false, false, false);
            for permission in permissions.iter() {
                match permission.r#type {
                    PermissionEnum::Read => p.0 = true,
                    PermissionEnum::Create | PermissionEnum::Update => p.1 = true,
                    PermissionEnum::Delete => p.2 = true,
                }
            }

            let permissions = ProjectPermissions {
                read: p.0,
                write: p.1,
                delete: p.2,
            };
            (project, permissions)
        })
        .collect::<Vec<_>>();

    Ok((role, projects))
}

#[server]
pub async fn update_role_permissions(
    role_name: String,
    permissions_to_grant: String,
    permissions_to_revoke: String,
) -> Result<(), ServerFnError> {
    use super::models::Permission;
    use crate::{database::Repo, server::AppState};
    use entity::sea_orm_active_enums::PermissionEnum;
    use leptos::use_context;

    let Some(state) = use_context::<AppState>() else {
        tracing::error!("Failed to get app state");
        return Err(ServerFnError::ServerError(
            "Failed to get app state".to_string(),
        ));
    };

    let db = state.conn;

    let permissions_to_grant = if let Ok(permissions) =
        serde_json::from_str::<Vec<(i32, Permission)>>(&permissions_to_grant)
    {
        permissions
            .iter()
            .map(|(id, permission)| {
                let r#type = match permission {
                    Permission::Read => PermissionEnum::Read,
                    Permission::Write => PermissionEnum::Create,
                    Permission::Delete => PermissionEnum::Delete,
                };
                (*id, r#type)
            })
            .collect::<Vec<_>>()
    } else {
        tracing::error!("Error parsing permissions to grant");
        return Err(ServerFnError::ServerError(
            "Error parsing permissions to grant".to_owned(),
        ));
    };

    let permissions_to_revoke = if let Ok(permissions) =
        serde_json::from_str::<Vec<(i32, Permission)>>(&permissions_to_revoke)
    {
        permissions
            .iter()
            .map(|(id, permission)| {
                let r#type = match permission {
                    Permission::Read => PermissionEnum::Read,
                    Permission::Write => PermissionEnum::Create,
                    Permission::Delete => PermissionEnum::Delete,
                };
                (*id, r#type)
            })
            .collect::<Vec<_>>()
    } else {
        tracing::error!("Error parsing permissions to revoke");
        return Err(ServerFnError::ServerError(
            "Error parsing permissions to revoke".to_owned(),
        ));
    };

    dbg!(&permissions_to_grant);
    dbg!(&permissions_to_revoke);

    if !permissions_to_grant.is_empty() {
        let _ = db
            .role_permissions()
            .create_many_for_role(&role_name, permissions_to_grant)
            .await;
    }

    if !permissions_to_revoke.is_empty() {
        let _ = db
            .role_permissions()
            .delete_many_for_role(&role_name, permissions_to_revoke)
            .await;
    }

    Ok(())
}
