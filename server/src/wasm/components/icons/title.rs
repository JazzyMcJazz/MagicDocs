use leptos::{component, view, IntoView};

#[component]
pub fn TitleIcon(
    #[prop(default = "100%")] size: &'static str,
    #[prop(default = "#e5e7eb")] color: &'static str,
) -> impl IntoView {
    view! {
        <svg width=size height=size viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
            <path
                d="M3 2a1 1 0 00-1 1v3a1 1 0 002 0 2 2 0 012-2h2v10.999A1 1 0 017 16h-.001A1 1 0 007 18h6a1 1 0 100-2 1 1 0 01-1-1V4h2a2 2 0 012 2 1 1 0 102 0V3a1 1 0 00-1-1H3z"
                fill=color
            />
        </svg>
    }
}
