use leptos::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <div class="relative flex justify-center items-center w-full min-h-[calc(100dvh-3.5rem)]">
            <div class="absolute font-bold text-center text-4xl animate-pulse">
                <p>"Work in"</p>
                <p>"Progress"</p>
            </div>
        </div>
    }
}
