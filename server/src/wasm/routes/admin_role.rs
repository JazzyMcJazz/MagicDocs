use leptos::{event_target_checked, *};
use leptos_router::use_params;

use crate::{
    server_functions::{
        get_role,
        models::{Permission, ProjectData, ProjectPermissions},
        update_role_permissions,
    },
    wasm::types::AdminParams,
};

#[component]
pub fn AdminRole() -> impl IntoView {
    let params = use_params::<AdminParams>();

    let role_name = move || {
        with!(|params| params
            .as_ref()
            .map(|params| params.role_name())
            .unwrap_or_default()
            .unwrap_or_default())
    };

    let data = create_resource(
        move || role_name(),
        move |role_name| async move { get_role(role_name).await },
    );

    let permissions = create_rw_signal(Vec::<(ProjectData, ProjectPermissions)>::new());
    let permissions_init_state = create_rw_signal(Vec::<(ProjectData, ProjectPermissions)>::new());

    let on_change = move |i: i32, r#type: Permission, checked: bool| {
        permissions.update(|permissions| {
            let (_, permission) = permissions
                .iter_mut()
                .find(|(project, _)| project.id == i)
                .unwrap();

            match r#type {
                Permission::Read => permission.read = checked,
                Permission::Write => permission.write = checked,
                Permission::Delete => permission.delete = checked,
            }
        });
    };

    let save = move |_| {
        let role_name = role_name();
        let mut permissions_to_grant = Vec::new();
        let mut permissions_to_revoke = Vec::new();
        let init_state = permissions_init_state.get();

        // Compare the current state of permissions with the initial state and add the changes to the respective vectors
        for (i, (project, permissions)) in permissions.get().iter().enumerate() {
            if permissions.read != init_state[i].1.read {
                if permissions.read {
                    permissions_to_grant.push((project.id, Permission::Read));
                } else {
                    permissions_to_revoke.push((project.id, Permission::Read));
                }
            }
            if permissions.write != init_state[i].1.write {
                if permissions.write {
                    permissions_to_grant.push((project.id, Permission::Write));
                } else {
                    permissions_to_revoke.push((project.id, Permission::Write));
                }
            }
            if permissions.delete != init_state[i].1.delete {
                if permissions.delete {
                    permissions_to_grant.push((project.id, Permission::Delete));
                } else {
                    permissions_to_revoke.push((project.id, Permission::Delete));
                }
            }
        }

        spawn_local(async move {
            let permissions_to_grant = serde_json::to_string(&permissions_to_grant).unwrap();
            let permissions_to_revoke = serde_json::to_string(&permissions_to_revoke).unwrap();

            let _ = update_role_permissions(role_name, permissions_to_grant, permissions_to_revoke)
                .await;
            data.refetch();
        });
    };

    create_effect(move |_| {
        if let Some(Ok((_, p))) = data.get() {
            permissions.set(p.to_owned());
            permissions_init_state.set(p);
        }
    });

    view! {
        <div class="p-10">
            <Transition fallback=|| ()>
                {move || match data.get() {
                    Some(Ok((role, _))) => Some(view! {
                        <h1>{ role.name }</h1>
                        <h4>{ role.description }</h4>
                        <hr class="mb-8" />

                        <div class="flex items-center gap-4 mb-4">
                            <h2 >"Project Permissions"</h2>
                            <input on:click=save type="submit" value="Save" class="btn-primary cursor-pointer"/>
                        </div>
                        <table id="permission-table">
                            <thead>
                                <tr>
                                    <th>Project</th>
                                    <th>Read</th>
                                    <th>Write</th>
                                    <th>Delete</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || permissions.get().to_owned()
                                    key=|(project, _)| project.id.to_owned()
                                    let:data
                                >
                                    <tr>
                                        <td>{ data.0.name }</td>
                                        <td><input
                                            type="checkbox"
                                            name="read"
                                            checked=data.1.read
                                            on:change=move |e| on_change(data.0.id, Permission::Read, event_target_checked(&e))
                                        /></td>
                                        <td><input
                                            type="checkbox"
                                            name="write"
                                            checked=data.1.write
                                            on:change=move |e| on_change(data.0.id, Permission::Write, event_target_checked(&e))
                                        /></td>
                                        <td><input
                                            type="checkbox"
                                            name="delete"
                                            checked=data.1.delete
                                            on:change=move |e| on_change(data.0.id, Permission::Delete, event_target_checked(&e))
                                        /></td>
                                    </tr>
                                </For>
                            </tbody>
                        </table>

                        <input on:click=save type="submit" value="Save" class="btn-primary cursor-pointer mt-4"/>
                    }),
                    _ => None,
                }}
            </Transition>
        </div>
    }
}
