use leptos::*;

use crate::server_functions::get_user_list;

#[component]
pub fn AdminUsers() -> impl IntoView {
    let users = create_resource(|| (), move |_| async { get_user_list().await });

    view! {
        <div class="p-10">
            <h1>Users</h1>
            <hr/>

            <table id="role-table" class="mt-8">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Email</th>
                        <th>Verified</th>
                    </tr>
                </thead>
                <tbody>
                    <Suspense fallback=|| ()>
                        {move || match users.get() {
                            Some(Ok(users)) => Some(view! {
                                <For
                                    each=move || users.to_owned()
                                    key=|user| user.id.to_owned()
                                    let:user
                                >
                                    <tr>
                                        <td>
                                            <a href=format!("/admin/users/{}", user.id)>{ user.first_name } { user.last_name }</a>
                                        </td>
                                        <td>
                                            <a href=format!("/admin/users/{}", user.id)>{ user.email }</a>
                                        </td>
                                        <td>
                                            <a href=format!("/admin/users/{}", user.id)>
                                                { if user.email_verified {
                                                    "Yes"
                                                } else {
                                                    "No"
                                                }}
                                            </a>
                                        </td>
                                    </tr>
                                </For>
                             }),
                            _ => None,
                        }}
                    </Suspense>
                </tbody>
            </table>
        </div>
    }
}
