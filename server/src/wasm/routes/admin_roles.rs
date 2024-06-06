use leptos::*;
use leptos_router::ActionForm;

use crate::server_functions::{get_role_list, CreateRole};

#[component]
pub fn AdminRoles() -> impl IntoView {
    let roles = create_resource(|| (), move |_| async { get_role_list().await });

    let name_ref = create_node_ref::<html::Input>();
    let description_ref = create_node_ref::<html::Input>();

    let create_role = create_server_action::<CreateRole>();

    let result = create_role.value();

    create_effect(move |_| {
        if let Some(Ok(_)) = result.get() {
            roles.refetch();
        }
    });

    create_effect(move |_| {
        if let Some(Ok(_)) = roles.get() {
            name_ref.get().unwrap().set_value("");
            description_ref.get().unwrap().set_value("");
        }
    });

    view! {
        <div class="p-10">
            <h1>"Roles"</h1>
            <hr/>

            <div class="mt-8">
                <h3>"Create new role"</h3>
                <ActionForm action=create_role>
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label for="name">"Role Name"</label>
                            <input
                                ref=name_ref
                                type="text"
                                name="name"
                                id="name"
                                class="w-full p-2 border border-gray-300 rounded"
                                required
                            />
                        </div>
                        <div>
                            <label for="description">"Description"</label>
                            <input
                                ref=description_ref
                                type="text"
                                name="description"
                                id="description"
                                class="w-full p-2 border border-gray-300 rounded"
                            />
                        </div>
                        <div>
                            <button class="btn-primary">"Create Role"</button>
                        </div>
                    </div>
                </ActionForm>
            </div>

            <table id="role-table" class="mt-8">
                <thead>
                    <tr>
                        <th>"Role Name"</th>
                        <th>"Description"</th>
                    </tr>
                </thead>
                <tbody>
                    <Transition fallback=|| ()>
                        <For
                            each=move || roles.get().unwrap_or(Ok(Vec::new())).unwrap_or_default()
                            key=|role| role.id.to_owned()
                            let:role
                        >
                            <tr>
                                <td>
                                    <a href=format!("/admin/roles/{}", &role.name)>{ &role.name }</a>
                                </td>
                                <td>
                                    <a href=format!("/admin/roles/{}", &role.name)>{ role.description }</a>
                                </td>
                            </tr>
                        </For>
                    </Transition>
                </tbody>
            </table>
        </div>
    }
}
