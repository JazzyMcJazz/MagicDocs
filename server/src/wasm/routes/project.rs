use std::ops::Not as _;

use leptos::*;

use crate::wasm::{components::FinalizeButton, types::ProjectDataContext};

#[component]
/// Route component which redirects to the latest version of a project.
pub fn Project() -> impl IntoView {
    let Some((project, _)) = use_context::<ProjectDataContext>() else {
        return view! {<div class="p-10"><h1>"Loading..."</h1></div>
        };
    };

    view! {
        <div class="p-10">
            <div class="flex justify-between items-center">
                <div>
                    <h1>{project.name}</h1>
                    <p>{project.description}</p>
                </div>

                {move || project.finalized.not().then(|| {
                    view! { <FinalizeButton /> }
                })}
            </div>

            <hr class="my-8"/>
        </div>
    }
}
