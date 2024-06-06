use crate::{
    server_functions::CreateProject,
    wasm::{
        components::icons::{PenIcon, TitleIcon},
        types::ProjectsResource,
    },
};
use leptos::*;
use leptos_meta::Title;
use leptos_router::*;

#[component]
pub fn ProjectNew() -> impl IntoView {
    let projects = use_context::<ProjectsResource>().expect("Projects context not found");
    let navigate = use_navigate();

    let create_project = create_server_action::<CreateProject>();

    // Contains the return value of the server function
    let result = create_project.value();

    // React to changes in the result value
    create_effect(move |_| {
        if let Some(value) = result.get() {
            match value {
                Ok(id) => {
                    projects.refetch();
                    navigate(&format!("/projects/{}", id), Default::default());
                }
                Err(e) => logging::error!("Error creating project: {:?}", e),
            }
        }
    });

    view! {
        <Title text="Magic Docs - New Project"/>

        <div class="p-10">
            <h1>"Create new Project"</h1>
            <p>"Projects are the way to organize your documentation"</p>

            <hr class="my-8"/>

            <div class="grid grid-cols-base gap-y-4 gap-x-8 max-w-[34rem]">
                <ActionForm action=create_project>

                    <label for="project_name">"Project Name"</label>
                    <div class="flex items-center">
                        <div

                            class="px-2 w-10 h-full bg-[#202020] border-l-2 border-t-2 border-b-2 border-base rounded-l-sm"
                        >
                            <TitleIcon />
                        </div>
                        <input
                            type="text"
                            class="!rounded-l-none w-full"
                            id="project_name"
                            name="project_name"
                            autocomplete="off"
                            required
                        />
                    </div>

                    <label for="description">Description</label>
                    <div class="flex items-center">
                        <div class="px-2 w-10 h-full bg-[#202020] border-l-2 border-t-2 border-b-2 border-base rounded-l-sm">
                            <PenIcon />
                        </div>
                        <textarea
                            class="!rounded-l-none w-full min-h-28 max-h-96"
                            id="description"
                            name="description"
                            autocomplete="off"
                        ></textarea>
                    </div>

                    <button type="submit" class="btn-primary absolute bottom-8 w-fit">"Create Project"</button>
                </ActionForm>
            </div>
        </div>
    }
}
