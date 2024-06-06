use ev::SubmitEvent;
use leptos::*;

#[component]
/// Route component which redirects to the latest version of a project.
pub fn Editor<F>(
    header: &'static str,
    title: RwSignal<String>,
    content: RwSignal<String>,
    on_submit: F,
) -> impl IntoView
where
    F: Fn(SubmitEvent) + 'static,
{
    view! {
        <div class="relative flex-grow flex flex-col gap-4 p-8">
            <h1 class="text-2xl font-bold text-white">{header}</h1>
            <form class="flex flex-col" on:submit=on_submit>
                <label for="document-title" class="text-white mt-8 ml-1 font-bold">Title</label>
                <input
                    on:input=move |e| title.set(event_target_value(&e))
                    id="document-title"
                    type="text"
                    class="w-full p-4 mb-8 text-white bg-[#181818] border-2 focus:ring-0 focus:border-pink-500/50"
                    placeholder="Document Title"
                    name="title"
                    required
                />

                <label for="textarea" class="text-white mt-8 ml-1 font-bold">Content</label>

                <div id="editor-wrapper">
                    <textarea
                        on:input=move |e| content.set(event_target_value(&e))
                        onInput="this.parentNode.dataset.replicatedValue = this.value"
                        name="content"
                        id="textarea"
                        class="min-h-80"
                    ></textarea>
                </div>

                <button type="submit" class="absolute top-8 right-8 btn-primary w-fit bg-green-900">Save</button>
            </form>
        </div>
    }
}
