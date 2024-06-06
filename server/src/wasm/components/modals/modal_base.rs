use ev::MouseEvent;
use leptos::*;

#[component]
pub fn ModalBase<F>(on_request_close: F, children: Children) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
{
    view! {
        <div
            class="fixed inset-0 flex justify-center items-center bg-black/50 z-50"
            on:click=on_request_close
        >
            <div class="modal-body">
                {children()}
            </div>
        </div>
    }
}
