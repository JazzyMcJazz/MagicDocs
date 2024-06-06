use leptos::*;
use leptos_router::*;

use crate::{
    server_functions::get_document,
    wasm::{
        components::DocumentContent,
        types::{ProjectDataContext, ProjectParams},
    },
};

#[component]
/// Route component which redirects to the latest version of a project.
pub fn Document() -> impl IntoView {
    let params = use_params::<ProjectParams>();

    let Some(project_data) = use_context::<ProjectDataContext>() else {
        return view! { <div>"No project data found"</div> };
    };

    let document_id = move || {
        with!(|params| {
            params
                .as_ref()
                .map(|params| params.document_id())
                .unwrap_or_default()
                .unwrap_or_default()
        })
    };

    let is_finalized = move || project_data.0.finalized;

    let document = create_blocking_resource(
        move || (project_data.to_owned(), document_id()),
        move |((project, _), document_id)| async move {
            get_document(project.id, project.version, document_id).await
        },
    );

    view! {
        <div id="doc-parent" class="relative p-10">
            <Transition fallback=|| ()>
                {move || document.get().and_then(|document| match document {
                    Ok(document) => Some(view! { <DocumentContent document finalized=is_finalized() /> }),
                    _ => None,
                })}
            </Transition>
        </div>
    }
}
