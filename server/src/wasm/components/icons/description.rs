use leptos::{component, view, IntoView};

#[component]
pub fn DescriptionIcon() -> impl IntoView {
    view! {
        <svg
            fill="#e5e7eb"
            width="100%"
            height="100%"
            viewBox="0 0 24 24"
        >
            <path d="M21,7H3V4A1,1,0,0,1,4,3H20a1,1,0,0,1,1,1ZM3,20V9H21V20a1,1,0,0,1-1,1H4A1,1,0,0,1,3,20Zm3-6H18V12H6Zm0,4h6V16H6Z"/>
        </svg>
    }
}
