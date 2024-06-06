use crate::wasm::{
    components::{modals::ModalController, AppError, ErrorTemplate, RouteOutlet},
    layout::Layout,
    routes::*,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/magicdocs.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Title text="Magic Docs"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <ModalController>
                <Routes>
                    <Route path="" view=Layout>
                        <Route path="" view=Home/>
                        <Route path="/admin" view=AdminLayout>
                            <Route path="" view=AdminDashboard/>
                            <Route path="/users" view=RouteOutlet>
                                <Route path=":user_id" view=AdminUser/>
                                <Route path="" view=AdminUsers/>
                            </Route>
                            <Route path="/roles" view=RouteOutlet>
                                <Route path=":role_name" view=AdminRole/>
                                <Route path="" view=AdminRoles/>
                            </Route>
                        </Route>
                        <Route path="/projects" view=RouteOutlet>
                            <Route path="/new" view=ProjectNew/>
                            <Route path=":project_id" view=ProjectLayout>
                                <Route path="/create" view=DocumentNew/>
                                <Route path="/documents" view=RouteOutlet>
                                    <Route path=":document_id" view=Document/>
                                </Route>
                                <Route path="" view=Project/>
                            </Route>
                        </Route>
                    </Route>
                </Routes>
            </ModalController>
        </Router>
    }
}
