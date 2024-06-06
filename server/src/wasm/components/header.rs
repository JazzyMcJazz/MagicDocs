use leptos::*;
use leptos_router::use_location;
use leptos_use::on_click_outside;

use crate::{
    server_functions::models::AppData,
    wasm::components::icons::{PersonCircleIcon, TriangleDownIcon},
};

#[component]
pub fn Header() -> impl IntoView {
    let Some(app_data) = use_context::<AppData>() else {
        return view! { <header></header> };
    };

    let location = use_location();
    let path = move || {
        if location.search.get().is_empty() {
            location.pathname.get()
        } else {
            format!("{}?{}", location.pathname.get(), location.search.get())
        }
    };

    let (show_menu, set_show_menu) = create_signal(false);

    let menu = create_node_ref::<html::Div>();

    let _ = on_click_outside(menu, move |_| set_show_menu.set(false));

    view! {
        <header class="fixed w-full h-14 flex items-center border-b-1 border-base z-10">
            <nav class="w-full h-full">
                <div class="flex gap-2 items-center justify-between px-2 h-full">
                    <div>
                        <a
                            id="nav-logo"
                            href="/"
                            class="flex items-center gap-2 text-xl"
                        >
                            <img src="/img/logo.png" alt="Magic Docs" class="h-12 p-1 rounded-md" />
                            "Magic Docs"
                        </a>
                    </div>

                    <div node_ref=menu class="relative flex gap-8 items-center mr-2 h-full">
                        <button
                            on:click=move |_| set_show_menu.update(|x| *x = !*x)
                            id="nav-user-menu-btn"
                            class="font-bold flex gap-1 items-center h-full px-2"
                        >

                            {format!("{} {}", app_data.user.given_name, app_data.user.family_name)}

                            <div class="w-5 mt-1">
                                <TriangleDownIcon />
                            </div>
                        </button>
                        <div class="w-10">
                            <PersonCircleIcon />
                        </div>

                        // Menu

                        <div
                            id="nav-user-menu"
                            class=move || if show_menu.get() {
                                "absolute right-0 top-12 bg-neutral-100 border-1 border-base rounded-md shadow-md py-2"
                            } else {
                                "hidden"
                            }
                        >
                            <form method="POST" action="/auth/refresh">
                                <input type="hidden" name="path" value={move || path()} />
                                <input type="submit" value="Flush Permissions" class="nav-user-menu-item" />
                            </form>

                            {move || app_data.user
                                    .is_super_admin
                                    .then(|| Some(view! { <><a href="/admin" class="nav-user-menu-item">"Manage"</a></> }))
                            }

                            <form method="POST" action="/auth/logout">
                                <input type="hidden" name="path" value={move || path()} />
                                <input type="submit" value="Logout" class="nav-user-menu-item" />
                            </form>
                        </div>
                    </div>
                </div>
            </nav>
        </header>
    }
}
