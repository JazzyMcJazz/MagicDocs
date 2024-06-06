use leptos::{component, view, IntoView};

#[component]
pub fn HomeIcon(
    #[prop(default = "100%")] size: &'static str,
    #[prop(default = "#fff")] color: &'static str,
) -> impl IntoView {
    view! {
        <svg fill=color width=size height=size viewBox="0 0 32 32" xmlns="http://www.w3.org/2000/svg">
            <path d="M27 18.039L16 9.501 5 18.039V14.56l11-8.54 11 8.538v3.481zm-2.75-.31v8.251h-5.5v-5.5h-5.5v5.5h-5.5v-8.25L16 11.543l8.25 6.186z"/>
        </svg>
    }
}
