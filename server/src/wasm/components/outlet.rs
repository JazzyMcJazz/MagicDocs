use leptos::*;
use leptos_router::Outlet;

#[component]
pub fn RouteOutlet() -> impl IntoView {
    view! { <Outlet/> }
}
