use std::ops::Not as _;

use ev::MouseEvent;
use leptos::*;
use leptos_meta::Title;
use leptos_router::*;

use crate::{
    server_functions::{delete_document, get_project_data},
    wasm::{
        components::{icons::*, *},
        types::{ProjectParams, VersionQuery},
    },
};

#[component]
pub fn ProjectLayout() -> impl IntoView {
    let params = use_params::<ProjectParams>();
    let query = use_query::<VersionQuery>();

    let project_id = move || {
        with!(|params| params
            .as_ref()
            .map(|params| params.project_id())
            .unwrap_or_default()
            .unwrap_or_default())
    };

    let version = move || {
        with!(|query| {
            query
                .as_ref()
                .map(|query| query.version())
                .unwrap_or_default()
        })
    };

    let project_data = create_blocking_resource(
        move || (project_id(), version()),
        move |(id, version)| async move { get_project_data(id, version).await },
    );

    let select_ref = create_node_ref::<html::Select>();

    let on_version_select = move |_| {
        let target = select_ref.get().expect("select_ref not found");
        let form = target.form().unwrap();
        let _ = form.submit();
    };

    provide_context(project_data);

    view! {
        <Transition>
            {move || match project_data.get() {
                Some(Ok((project, _))) => Some(view! {
                    <Title text=format!("Magic Docs - {}", project.name)/>
                }),
                _ => None,
            }}
        </Transition>

        <div class="relative flex h-[calc(100dvh-3.5rem)]">
            <div id="document-nav" class="w-48 h-[calc(100dvh-3.5rem)] bg-[#181818] border-r-1 border-base">
                // TODO: {% if permissions.write %}
                <div class="flex justify-center w-full p-4 border-b-1 border-base">
                    <A
                        href="create"
                        class="btn-primary whitespace-nowrap"
                    >
                        "New Document"
                    </A>
                </div>

                // Project Version Selector
                <Suspense fallback=|| ()>
                    {move || match project_data.get() {
                        Some(Ok((project, _))) => if project.versions.len() > 1 {
                            Some(view! {
                                <Form method="GET" action="">
                                    <select
                                        id="document-filter"
                                        class="w-full p-3 bg-[#181818] text-white"
                                        name="version"
                                        ref=select_ref
                                        on:change=on_version_select
                                    >
                                        <For
                                            each=move || project.versions.to_owned()
                                            key=|version| version.to_owned()
                                            let:value
                                        >
                                            <option
                                                value=value
                                                selected=move || if let Some(version) = version() {
                                                    value.eq(&version)
                                                } else {
                                                    false
                                                }
                                            >
                                                "Version " { value }
                                            </option>
                                        </For>
                                    </select>
                                </Form>
                            })
                        } else {
                            None
                        },
                        _ => None,
                    }}
                </Suspense>

                // Document List Sidebar
                <ul class="max-h-[calc(100dvh-3.5rem-75px)] overflow-y-auto">
                    <Suspense fallback=|| ()>
                        {move || match project_data.get() {
                            Some(Ok((project, documents))) => {
                                let navigate = use_navigate();
                                let redirect_url = create_rw_signal(None::<String>);

                                let is_latest_version = project.version
                                    .eq(&project.versions.into_iter().max().unwrap());

                                // On delete document
                                let on_delete = move |e: MouseEvent, doc_id: i32| {
                                    e.stop_propagation();

                                    let confirm = window()
                                        .confirm_with_message("Are you sure you want to delete this document?");
                                    match confirm {
                                        Ok(true) => (),
                                        _ => return,
                                    }

                                    spawn_local(async move {
                                        let Ok(_) = delete_document(project.id, project.version, doc_id).await else {
                                            logging::log!("Failed to delete document");
                                            return;
                                        };
                                        project_data.refetch();
                                        if let Ok(current_path) = window().location().pathname() {
                                            if current_path.eq(&format!("/projects/{}/documents/{}", project.id, doc_id)) {
                                                redirect_url.set(Some(format!("projects/{}", project.id)));
                                            }
                                        };
                                    });
                                };

                                // Handle redirect after document delete, if we are on the deleted document page
                                create_effect(move |_| {
                                    if let Some(url) = redirect_url.get() {
                                        navigate(url.as_str(), Default::default());
                                        redirect_url.set(None);
                                    }
                                });

                                Some(view! {
                                    <For
                                        each=move || documents.to_owned()
                                        key=|doc| doc.id
                                        let:doc
                                    >
                                        <li class="relative group flex items-center">
                                            <A
                                                href=move || if let Some(v) = version() {
                                                    format!("documents/{}?version={}", doc.id, v)
                                                } else {
                                                    format!("documents/{}", doc.id)
                                                }
                                                class="w-full"
                                                exact=true
                                            >
                                                <p>{doc.name}</p>

                                            </A>

                                            // TODO if is_latest_version and permissions.delete
                                            {move || is_latest_version.then(|| {
                                                view!{
                                                    <button
                                                        on:click=move |e| on_delete(e, doc.id)
                                                        class="absolute right-2 opacity-0 group-hover:opacity-60 w-5 h-5 text-red-500"
                                                    >
                                                        <TrashIcon />
                                                    </button>
                                                }
                                            })}
                                        </li>
                                    </For>
                                })
                            },
                            _ => None,
                        }}
                    </Suspense>

                </ul>

            </div>

            // Outlet and Version Draft Banner
            <div class="relative flex-grow float-end w-[calc(100dvw-25rem)] overflow-y-auto">
                <Suspense fallback=|| ()>
                    {move || match project_data.get() {
                        Some(Ok((project, _))) => project.finalized.not().then(|| Some(view! {
                            <div class="absolute top-0 left-0 right-0 h-6 bg-yellow-600">
                                <p class="text-center text-white font-bold">"Project Version Draft"</p>
                            </div>
                        })),
                        _ => None,
                    }}
                </Suspense>

                <Transition>
                    {move || match project_data.get() {
                        Some(Ok(data)) => {
                            provide_context(data);
                            Some(view! { <Outlet/> })
                        },
                        _ => None,
                    }}
                </Transition>
            </div>

            <Transition>
                {move || match project_data.get() {
                    Some(Ok((project, _))) => if project.finalized {
                        Some(view! { <ChatPanel project /> })
                    } else {
                        None
                    },
                    _ => None,
                }}
            </Transition>

            // {% endif %}"
        </div>
    }
}
