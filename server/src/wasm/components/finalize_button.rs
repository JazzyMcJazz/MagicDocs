use futures_util::StreamExt;
use leptos::*;

use crate::{
    server_functions::finalize_project_version,
    wasm::types::{ProjectDataContext, ProjectDataResource},
};

use super::modals::ModalType;

#[component]
pub fn FinalizeButton() -> impl IntoView {
    let Some((project, _)) = use_context::<ProjectDataContext>() else {
        return view! { <></> }.into_view();
    };

    let Some(project_data) = use_context::<ProjectDataResource>() else {
        return view! { <></> }.into_view();
    };

    let Some(modal_type) = use_context::<RwSignal<Option<ModalType>>>() else {
        return view! { <></> }.into_view();
    };

    let on_click = move |_| {
        modal_type.set(Some(ModalType::Finalizing("Starting...".to_string())));

        spawn_local(async move {
            let mut stream = finalize_project_version(project.id, project.version)
                .await
                .expect("Failed to finalize project version")
                .into_inner();

            while let Some(Ok(chunk)) = stream.next().await {
                let message = chunk.trim_start_matches("data: ").trim_end_matches("¤¤");

                modal_type.set(Some(ModalType::Finalizing(message.to_string())));
            }

            project_data.refetch();
            modal_type.set(None);
        });
    };

    view! {
        // TODO: {% if not is_finalized and permissions.write and documents | length > 0 %}
        <button
            class="btn-primary cursor-pointer"
            on:click=on_click
        >
            "Finalize Version"
        </button>
    }
    .into_view()
}
