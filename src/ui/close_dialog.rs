use dioxus::prelude::*;

#[component]
pub fn CloseDialog(
    left_dirty: bool,
    right_dirty: bool,
    on_save_and_close: EventHandler<()>,
    on_close: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let msg = if left_dirty && right_dirty {
        "Both files have unsaved changes."
    } else if left_dirty {
        "The left file has unsaved changes."
    } else {
        "The right file has unsaved changes."
    };

    rsx! {
        div { class: "modal-overlay",
            div { class: "modal-dialog",
                p { class: "modal-message", "{msg}" }
                div { class: "modal-buttons",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| on_save_and_close.call(()),
                        "Save & Close"
                    }
                    button {
                        class: "btn btn-danger",
                        onclick: move |_| on_close.call(()),
                        "Close without saving"
                    }
                    button { class: "btn", onclick: move |_| on_cancel.call(()), "Cancel" }
                }
            }
        }
    }
}
