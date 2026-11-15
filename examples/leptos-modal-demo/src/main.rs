//! Minimal Leptos CSR + WASM example using `glastor-aria-core` and `glastor-css-binder`.

use glastor_aria_core::adapter::{FnSink, FnState, ReactiveFsm};
use glastor_aria_core::primitives::dialog::{
    DialogAction, DialogEvent, DialogFsm, DialogState, DomDialogEffect,
};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Executes a [`DomDialogEffect`] against the browser DOM via `web-sys`.
fn apply_dom_effect(effect: DomDialogEffect<'_>) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    match effect {
        DomDialogEffect::FocusById(id) => {
            if let Some(el) = doc.get_element_by_id(id)
                && let Ok(html) = el.dyn_into::<web_sys::HtmlElement>()
            {
                let _ = html.focus();
            }
        }
        DomDialogEffect::SetBodyOverflow(overflow) => {
            if let Some(body) = doc.body() {
                let _ = body.style().set_property("overflow", overflow);
            }
        }
        DomDialogEffect::None => {}
    }
}

#[component]
fn App() -> impl IntoView {
    let state_signal = RwSignal::new(DialogState::Closed);
    let fsm = ReactiveFsm::new(
        DialogFsm::new("strata-modal"),
        FnState::new(
            move || state_signal.get(),
            move |next| state_signal.set(next),
        ),
        FnSink::new(|action: DialogAction| {
            apply_dom_effect(action.to_dom_effect("strata-modal-trigger", "strata-modal-close"));
        }),
    );

    let is_open = move || fsm.state() == DialogState::Open;
    let fsm_open = fsm.clone();
    let fsm_close = fsm.clone();
    let fsm_backdrop = fsm.clone();
    let fsm_keydown = fsm.clone();

    view! {
        <main class="flex items-center justify-center p-6">
            <div class="flex flex-col items-center gap-4 p-6 rounded-lg">
                <h1 class="text-2xl font-bold">"GLASTOR® Strata × Leptos"</h1>
                <p class="text-sm text-muted-foreground">
                    "Zero-Node.js CSS + WAI-ARIA 1.2 deterministic Dialog FSM"
                </p>
                <button
                    id="strata-modal-trigger"
                    type="button"
                    aria-haspopup="dialog"
                    aria-expanded=move || if is_open() { "true" } else { "false" }
                    aria-controls="strata-modal"
                    class="inline-flex items-center justify-center px-4 py-2 rounded-md font-medium bg-primary text-primary-foreground hover:bg-slate-800 focus-visible:outline-none focus-visible:ring-2"
                    on:click=move |_| {
                        fsm_open.send(DialogEvent::OpenTriggerClicked);
                    }
                >
                    "Open Accessible Modal"
                </button>
            </div>

            <Show when=is_open>
                <div
                    class="modal-backdrop flex items-center justify-center p-4"
                    on:click=move |_| {
                        fsm_backdrop.send(DialogEvent::OutsidePointerDown);
                    }
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Escape" {
                            ev.prevent_default();
                            fsm_keydown.send(DialogEvent::EscapePressed);
                        }
                    }
                >
                    <div
                        id="strata-modal"
                        role="dialog"
                        aria-modal="true"
                        aria-labelledby="strata-modal-title"
                        aria-describedby="strata-modal-description"
                        class="modal-card p-6 rounded-lg flex flex-col gap-4"
                        on:click=|ev| ev.stop_propagation()
                    >
                        <h2 id="strata-modal-title" class="text-lg font-semibold">
                            "Zero-Node Accessible Dialog"
                        </h2>
                        <p id="strata-modal-description" class="text-sm text-muted-foreground">
                            "Managed by glastor-aria-core DialogFsm. Press Escape, click outside, or click Close to restore focus."
                        </p>
                        <div class="flex justify-end gap-2">
                            <button
                                id="strata-modal-close"
                                type="button"
                                class="px-4 py-2 rounded-md font-medium bg-primary text-primary-foreground hover:bg-slate-800 focus-visible:outline-none focus-visible:ring-2"
                                on:click=move |_| {
                                    fsm_close.send(DialogEvent::CloseTriggerClicked);
                                }
                            >
                                "Close"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </main>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
