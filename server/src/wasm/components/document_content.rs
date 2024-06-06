use std::ops::Not as _;

use leptos::*;

use crate::{server_functions::models::Document, wasm::components::FinalizeButton};

#[component]
pub fn DocumentContent(document: Document, finalized: bool) -> impl IntoView {
    view! {
        <div class="flex justify-between items-center">
            <div class="flex gap-2 items-center">
                <h1>{document.name}</h1>
                // {% if is_latest_version and permissions.write %}
                <div
                    // hx-get="/projects/{{ project.id }}/v/{{ project_version }}/documents/{{ document.id }}/edit"
                    // hx-target="#doc-parent"
                    // hx-swap="innerHTML"
                    class="w-6 h-6 cursor-pointer"
                >
                // {% include 'icons/pen.svg' %}
                </div>
                // {% endif %}
            </div>

            {move || finalized.not().then(|| {
                view! { <FinalizeButton /> }
            })}
        </div>
        <hr class="my-8"/>
        <div id="document-content" inner_html=document.content></div>
    }
}
