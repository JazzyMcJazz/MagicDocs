use leptos::*;
use leptos_router::Outlet;

use crate::{
    server_functions::{get_app_data, get_projects},
    wasm::components::{Header, Sidebar},
};

#[component]
pub fn Layout() -> impl IntoView {
    let app_data = create_blocking_resource(|| (), |_| async move { get_app_data().await });

    let projects: Resource<
        (),
        Result<Vec<crate::server_functions::models::ProjectData>, ServerFnError>,
    > = create_blocking_resource(|| (), |_| async move { get_projects().await });

    provide_context(projects);

    view! {
        <Transition>
            {move || app_data.get().map(|data| match data {
                Ok(data) => {
                    provide_context(data);
                    Some(view! {

                            <Header/>
                            <Sidebar projects />

                            <main id="page" class="mt-14 w-[calc(100dvw-13rem)] float-end min-h-[calc(100dvh-3.5rem)]">
                                <Outlet/>
                            </main>

                    })
                },
                Err(_) => None,
            } )}
        </Transition>
    }
}
