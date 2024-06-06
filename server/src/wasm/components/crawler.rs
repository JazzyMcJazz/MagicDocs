use futures_util::StreamExt;
use leptos::*;

use crate::{
    server_functions::crawl_website,
    wasm::{components::icons::SpinnerIcon, types::ProjectDataResource},
};

#[component]
pub fn Crawler(project_id: i32) -> impl IntoView {
    let project_data =
        use_context::<ProjectDataResource>().expect("ProjectDataResource context not found");

    let url = create_rw_signal(String::new());
    let max_depth_enabled = create_rw_signal(false);
    let max_depth = create_rw_signal(String::new());
    let crawling = create_rw_signal(false);
    let output_message = create_rw_signal(String::new());

    let on_start = move |_| {
        if crawling.get() {
            return;
        }

        crawling.set(true);

        let url = url.get();
        let use_max_depth = max_depth_enabled.get();
        let max_depth = match use_max_depth {
            true => Some(max_depth.get().parse::<usize>().unwrap_or(0)),
            false => None,
        };

        spawn_local(async move {
            let mut stream = crawl_website(project_id, url, max_depth)
                .await
                .expect("Failed to start crawl stream")
                .into_inner();

            while let Some(Ok(output)) = stream.next().await {
                output_message.set(output);
            }

            project_data.refetch();

            crawling.set(false);
        });
    };

    view! {
        <div class="flex-grow flex flex-col gap-4 p-8">
            <div class="flex flex-col gap-1">
                <label for="url" class="text-lg">URL</label>
                <input
                    type="text"
                    name="url"
                    id="url"
                    placeholder="https://example.com"
                    required
                    on:input=move |e| url.set(event_target_value(&e))
                />
            </div>

            <div class="flex flex-col gap-1">
                <div class="flex gap-1 items-center mt-2">
                    <input
                        on:change=move |e| max_depth_enabled.set(event_target_checked(&e))
                        type="checkbox"
                        name="toggle-depth"
                        id="toggle-depth"
                    />
                    <label for="toggle-depth" class="text-lg mt-0">Set Max Depth</label>
                </div>

                <label for="depth" id="depth-label" class="text-lg disabled">Maximum Depth</label>
                <input
                    on:input=move |e| max_depth.set(event_target_value(&e))
                    type="number"
                    name="depth"
                    id="depth"
                    value="0"
                    min="0"
                    max="99"
                    class="max-w-24"
                    required
                    disabled
                />
            </div>

            <input
                on:click=on_start
                type="submit"
                value="Start"
                disabled=move || crawling.get()
                class="btn-primary cursor-pointer w-fit"
            />

            <div
                id="crawler-output-container"
                class=move || if crawling.get() {"items-center gap-4"} else {"hidden"}
            >
                <div class="w-8 h-8 animate-spin">
                    <SpinnerIcon />
                </div>
                <p class="text-gray-200">{ move || output_message.get() }</p>
            </div>
        </div>
    }
}
