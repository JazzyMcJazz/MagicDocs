use leptos::*;

use super::FinalizingModal;

#[derive(Debug, Clone)]
pub enum ModalType {
    Finalizing(String),
}

#[component]
pub fn ModalController(children: Children) -> impl IntoView {
    let modal_type = create_rw_signal(None::<ModalType>);
    // let modal_type = create_rw_signal(Some(ModalType::Finalizing(String::from("file.txt"))));
    provide_context(modal_type);

    view! {
        {move || match modal_type.get() {
            Some(ModalType::Finalizing(message)) => view! { <FinalizingModal message /> }.into_view(),
            None => view! { <></> }.into_view(),
        }}
        <>{children()}</>
    }
}
