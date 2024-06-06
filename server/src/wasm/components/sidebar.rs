use std::ops::Not as _;

use leptos::*;
use leptos_router::A;

use crate::{server_functions::models::AppData, wasm::types::ProjectsResource};

#[component]
pub fn Sidebar(projects: ProjectsResource) -> impl IntoView {
    let app_data = use_context::<AppData>().expect("AppData context not found");

    view! {
        <aside class="fixed top-14 w-52 h-[calc(100dvh-3.5rem)] border-r-1 border-base z-10">
            <div class="flex flex-col h-full">

                {move || app_data.user.is_admin.then(|| Some(view! {
                    <div class="w-full flex justify-center p-4">
                        <a
                            href="/projects/new"
                            class="btn-primary w-full"
                        >"New Project"</a>
                    </div>
                    <hr />
                }))}

                <Transition fallback=move || ()>
                    {move || match projects.get() {
                        Some(Ok(projects)) => projects.is_empty().not().then(|| view! {
                            <p class="text-sm my-4 ml-4">"Projects"</p>
                        }),
                        _ => None,
                    }}
                </Transition>

                <ul id="sidebar" class="overflow-y-auto">
                    <Transition fallback=move || ()>
                        {move || match projects.get() {
                            Some(Ok(data)) => Some(view!{
                                <For
                                    each=move || data.to_owned()
                                    key=|state| state.id
                                    let:child
                                >
                                    <li
                                        title=&child.name
                                        class="relative"
                                    >
                                        <A
                                            href=move || format!("/projects/{}", child.id)
                                            class="sidebar-item"
                                        >
                                            {child.name}
                                        </A>
                                        <span class="absolute top-0 right-0 w-8 h-full bg-gradient-to-l from-[#101010] to-transparent"
                                        ></span>
                                    </li>
                                </For>
                            }),
                            _ => None,
                        }}
                    </Transition>
                </ul>
            </div>
        </aside>
    }
}
