use leptos::*;

use crate::wasm::components::{icons::SpinnerIcon, modals::ModalBase};

#[component]
pub fn FinalizingModal(message: String) -> impl IntoView {
    logging::log!("FinalizingModal");

    view! {
        <ModalBase on_request_close=|_| {}>
            <div class="w-96">
                <h2 class="text-lg font-semibold">"Finalizing Project Version"</h2>
                <p class="text-sm text-gray-300 mb-2">"Please wait while the version is being finalized."</p>
                <div class="flex items-center">
                    <SpinnerIcon size="24px" color="#fff" />
                    <p class="ml-2">{message}</p>
                </div>

            </div>
        </ModalBase>
    }
}
