use leptos::{component, view, IntoView};

#[component]
pub fn SpinnerIcon(
    #[prop(default = "100%")] size: &'static str,
    #[prop(default = "#e5e7eb")] color: &'static str,
) -> impl IntoView {
    view! {
        <div class="animate-spin">
            <svg width=size height=size viewBox="0 0 14 14" xmlns="http://www.w3.org/2000/svg">
                <g fill="none" fill-rule="evenodd">
                    <circle cx="7" cy="7" r="6" stroke=color stroke-opacity="0.4" stroke-width="2"/>
                    <path fill=color fill-opacity=".8" fill-rule="nonzero" d="M7 0a7 7 0 0 1 7 7h-2a5 5 0 0 0-5-5V0z"/>
                </g>
            </svg>
        </div>
    }
}
