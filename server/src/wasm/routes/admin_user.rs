use leptos::*;
use leptos_router::use_params;

use crate::{
    server_functions::{
        get_user,
        models::{AppRole, AppUser, Permission, ProjectData, ProjectPermissions},
        update_user_privileges,
    },
    wasm::{components::icons::SpinnerIcon, types::AdminParams},
};

#[component]
pub fn AdminUser() -> impl IntoView {
    let params = use_params::<AdminParams>();

    let user_id = move || {
        with!(|params| params
            .as_ref()
            .map(|params| params.user_id())
            .unwrap_or_default()
            .unwrap_or_default())
    };

    let available_roles_select_ref = create_node_ref::<html::Select>();
    let assigned_roles_select_ref = create_node_ref::<html::Select>();

    let saving = create_rw_signal(false);
    let user = create_rw_signal(None::<AppUser>);
    let assigned_roles = create_rw_signal(Vec::<AppRole>::new());
    let available_roles = create_rw_signal(Vec::<AppRole>::new());
    let roles_init_state = create_rw_signal((Vec::<AppRole>::new(), Vec::<AppRole>::new()));
    let permissions = create_rw_signal(Vec::<(ProjectData, ProjectPermissions)>::new());
    let permissions_init_state = create_rw_signal(Vec::<(ProjectData, ProjectPermissions)>::new());

    let data = create_resource(
        move || user_id(),
        move |user_id| async { get_user(user_id).await },
    );

    let grant_roles = move |_| {
        let node = available_roles_select_ref
            .get()
            .expect("Failed to get available roles select");
        let options = node.selected_options();

        let mut selected = Vec::new();
        for i in 0..options.length() {
            let option = options.get_with_index(i).expect("Failed to get option");
            let value = option.get_attribute("value").expect("Failed to get value");
            selected.push(value);
        }

        let roles_to_grant = available_roles
            .get()
            .iter()
            .filter(|role| selected.contains(&role.id))
            .cloned()
            .collect::<Vec<_>>();

        available_roles.update(|roles| {
            *roles = roles
                .iter()
                .filter(|role| !roles_to_grant.contains(role))
                .cloned()
                .collect::<Vec<_>>();
        });

        assigned_roles.update(|roles| {
            roles.extend(roles_to_grant);
            roles.sort_by(|a, b| b.name.cmp(&a.name));
        });
    };

    let revoke_roles = move |_| {
        let node = assigned_roles_select_ref
            .get()
            .expect("Failed to get assigned roles select");
        let options = node.selected_options();

        let mut selected = Vec::new();
        for i in 0..options.length() {
            let option = options.get_with_index(i).expect("Failed to get option");
            let value = option.get_attribute("value").expect("Failed to get value");
            selected.push(value);
        }

        let roles_to_revoke = assigned_roles
            .get()
            .iter()
            .filter(|role| selected.contains(&role.id))
            .cloned()
            .collect::<Vec<_>>();

        assigned_roles.update(|roles| {
            *roles = roles
                .iter()
                .filter(|role| !roles_to_revoke.contains(role))
                .cloned()
                .collect::<Vec<_>>();
        });

        available_roles.update(|roles| {
            roles.extend(roles_to_revoke);
            roles.sort_by(|a, b| b.name.cmp(&a.name));
        });
    };

    let on_change_permission = move |i: i32, r#type: Permission, checked: bool| {
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
        saving.set(true);
        let user_id = user_id();

        let mut permissions_to_grant = Vec::new();
        let mut permissions_to_revoke = Vec::new();
        let permissions_init_state = permissions_init_state.get();
        let roles_init_state = roles_init_state.get();

        let roles_to_grant = assigned_roles
            .get()
            .iter()
            .filter(|role| !roles_init_state.0.contains(role))
            .cloned()
            .collect::<Vec<_>>();

        let roles_to_grant =
            serde_json::to_string(&roles_to_grant).expect("Failed to serialize roles to grant");

        let roles_to_revoke = available_roles
            .get()
            .iter()
            .filter(|role| !roles_init_state.1.contains(role))
            .cloned()
            .collect::<Vec<_>>();

        let roles_to_revoke =
            serde_json::to_string(&roles_to_revoke).expect("Failed to serialize roles to revoke");

        // Compare the current state of permissions with the initial state and add the changes to the respective vectors
        for (i, (project, permissions)) in permissions.get().iter().enumerate() {
            if permissions.read != permissions_init_state[i].1.read {
                if permissions.read {
                    permissions_to_grant.push((project.id, Permission::Read));
                } else {
                    permissions_to_revoke.push((project.id, Permission::Read));
                }
            }
            if permissions.write != permissions_init_state[i].1.write {
                if permissions.write {
                    permissions_to_grant.push((project.id, Permission::Write));
                } else {
                    permissions_to_revoke.push((project.id, Permission::Write));
                }
            }
            if permissions.delete != permissions_init_state[i].1.delete {
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
            let _ = update_user_privileges(
                user_id,
                roles_to_grant,
                roles_to_revoke,
                permissions_to_grant,
                permissions_to_revoke,
            )
            .await;
            data.refetch();
        });
    };

    create_effect(move |_| {
        if let Some(Ok((u, asr, avr, p))) = data.get() {
            user.set(Some(u));
            assigned_roles.set(asr.to_owned());
            available_roles.set(avr.to_owned());
            roles_init_state.set((asr, avr));
            permissions.set(p.to_owned());
            permissions_init_state.set(p);
        }
        saving.set(false);
    });

    view! {
        <div class="p-10">
            <Transition fallback=|| ()>
                {move || match data.get() {
                    Some(_) => Some(view!{
                        <div class="flex items-center justify-between">
                            <div>
                                { move || if let Some(user) = user.get() {
                                    view! {
                                        <h1>{ user.first_name} { user.last_name }</h1>
                                        <p class="text-sm">{ user.email }</p>
                                    }.into_view()
                                } else { view!{ <></> }.into_view() }}
                            </div>

                            {move || if saving.get() {
                                view!{ <div class="mr-4"><SpinnerIcon size="2.5rem" /></div> }.into_view()
                            } else {
                                view!{ <button on:click=save class="btn-primary">"Save"</button> }.into_view()
                            }}

                        </div>
                        <hr class="mb-8" />

                        <h2 class="mb-2">"User Roles"</h2>
                        <div class="flex gap-8">
                            <div class="flex-1">
                                <h3>"Available Roles"</h3>
                                <select
                                    ref=available_roles_select_ref
                                    id="available-roles-select"
                                    name="role_id"
                                    class="w-full bg-[#121212] p-0 min-h-44" multiple
                                    size="5"

                                >
                                    <For
                                        each=move || available_roles.get()
                                        key=|role| role.id.to_owned()
                                        let:role
                                    >

                                        <option
                                            value=role.id
                                            class="p-2"
                                        >
                                            { role.name }
                                        </option>
                                    </For>
                                </select>
                            </div>

                            <div class="flex flex-col gap-2 justify-center">
                                <button
                                    on:click=grant_roles
                                    id="assign-role-btn"
                                    class="btn-primary"
                                >Assign</button>
                                <button
                                    on:click=revoke_roles
                                    id="remove-role-btn"
                                    class="btn-primary"
                                >Remove</button>
                            </div>

                            <div class="flex-1">
                                <h3>"Assigned Roles"</h3>
                                <select
                                    ref=assigned_roles_select_ref
                                    id="assigned-roles-select"
                                    name="role_id"
                                    class="w-full bg-[#121212] p-0 min-h-44" multiple
                                    size="5"
                                >
                                    <For
                                        each=move || assigned_roles.get().to_owned()
                                        key=|role| role.id.to_owned()
                                        let:role
                                    >
                                        <option
                                            value=role.id
                                            class="p-2"
                                        >
                                            { role.name }
                                        </option>
                                    </For>
                                </select>
                            </div>
                        </div>

                        <hr class="my-8" />

                        <h2 class="mb-2">"User Level Project Permissions"</h2>
                        <form id="user-permissions-form">
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
                                                on:change=move |e| on_change_permission(data.0.id, Permission::Read, event_target_checked(&e))
                                            /></td>
                                            <td><input
                                                type="checkbox"
                                                name="write"
                                                checked=data.1.write
                                                on:change=move |e| on_change_permission(data.0.id, Permission::Write, event_target_checked(&e))
                                            /></td>
                                            <td><input
                                                type="checkbox"
                                                name="delete"
                                                checked=data.1.delete
                                                on:change=move |e| on_change_permission(data.0.id, Permission::Delete, event_target_checked(&e))
                                            /></td>
                                        </tr>
                                    </For>
                                </tbody>
                            </table>
                        </form>
                    }),
                    _=> None

                }}
            </Transition>
        </div>
    }
}
