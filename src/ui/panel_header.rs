use crate::types::{FileContent, Side};
use dioxus::prelude::*;

#[component]
pub fn PanelHeader(
    file: Option<FileContent>,
    side: Side,
    dirty: bool,
    on_open: EventHandler<()>,
    on_save: EventHandler<()>,
) -> Element {
    let (name, name_class) = match &file {
        Some(f) => (f.display_name(), "panel-filename"),
        None => ("(no file)".to_string(), "panel-filename unsaved"),
    };
    let open_label = if side == Side::Left {
        "Open L"
    } else {
        "Open R"
    };
    let save_label = if side == Side::Left {
        "Save L"
    } else {
        "Save R"
    };
    let can_save = file.as_ref().and_then(|f| f.path.as_ref()).is_some() && dirty;

    rsx! {
        div { class: "panel-header",
            span { class: "{name_class}", "{name}" }
            if can_save {
                button { class: "btn btn-save", onclick: move |_| on_save.call(()), "{save_label}" }
            }
            button { class: "btn", onclick: move |_| on_open.call(()), "{open_label}" }
        }
    }
}
