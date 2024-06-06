use ev::SubmitEvent;
use leptos::*;
use leptos_router::*;

use crate::{
    server_functions::create_document,
    wasm::{
        components::{Crawler, Editor},
        types::{ProjectDataResource, ProjectParams},
    },
};

#[component]
/// Route component which redirects to the latest version of a project.
pub fn DocumentNew() -> impl IntoView {
    let project_data =
        use_context::<ProjectDataResource>().expect("ProjectDataResource context not found");
    let params = use_params::<ProjectParams>();
    let navigate = use_navigate();

    let use_editor = create_rw_signal(true);
    let title = create_rw_signal(String::new());
    let content = create_rw_signal(String::new());
    let redirect_url = create_rw_signal(None::<String>);

    let project_id = move || {
        with!(|params| {
            params
                .as_ref()
                .map(|params| params.project_id())
                .unwrap_or_default()
                .unwrap_or_default()
        })
    };

    let on_submit_editor = move |e: SubmitEvent| {
        e.prevent_default();
        let project_id = project_id();
        let title = title.get();
        let content = content.get();
        spawn_local(async move {
            let Ok(document_id) = create_document(project_id, title, content).await else {
                logging::log!("Failed to create document");
                return;
            };
            project_data.refetch();
            let url = format!("/projects/{}/documents/{}", project_id, document_id);
            redirect_url.set(Some(url));
        });
    };

    create_effect(move |_| {
        if let Some(url) = redirect_url.get() {
            navigate(url.as_str(), Default::default());
        }
    });

    view! {
        <div class="flex items-end gap-8 w-full h-[75px] bg-[#181818] pl-8">
            <button
                on:click=move |_| use_editor.set(true)
                class=move || if use_editor.get() {"editor-tab active-editor-tab"} else {"editor-tab"}
            >
                Editor
            </button>

            <button
                on:click=move |_| use_editor.set(false)
                class=move || if use_editor.get() {"editor-tab"} else {"editor-tab active-editor-tab"}
            >
                Crawler
            </button>
        </div>

        {move || if use_editor.get() {
            view! {
                <Editor header="New Document" title content on_submit=on_submit_editor />
            }.into_view()
        } else {
            view! { <Crawler project_id=project_id() /> }.into_view()
        }}
    }
}
