use crate::wasm::components::icons::HomeIcon;
use leptos::*;
use leptos_router::*;

#[component]
pub fn AdminLayout() -> impl IntoView {
    view! {
        <div class="w-full h-16 bg-[#191919]">
            <div id="admin-nav" class="flex h-full">

                <A href="/admin" class="tab" exact=true>
                    <HomeIcon size="2rem" />
                </A>

                <A href="/admin/users" class="tab" exact=true>"Users"</A>
                <A href="/admin/roles" class="tab" exact=true>"Roles"</A>

                <div class="border-b-1 border-base flex-grow">

                </div>
            </div>
        </div>

        <div class="h-[calc(100dvh-7.3rem)] overflow-y-auto">
            <Outlet />
        </div>
    }
}
