use crate::types::{Theme, ViewMode};
use dioxus::prelude::*;

#[component]
pub fn Toolbar(
    theme: Theme,
    view_mode: ViewMode,
    ignore_ws: bool,
    on_toggle_theme: EventHandler<()>,
    on_toggle_view: EventHandler<()>,
    on_toggle_ws: EventHandler<()>,
) -> Element {
    let theme_label = if theme == Theme::Dark {
        "☀ Light"
    } else {
        "☾ Dark"
    };
    let view_label = if view_mode == ViewMode::Full {
        "Changes only"
    } else {
        "Full file"
    };
    let ws_class = if ignore_ws { "btn active" } else { "btn" };
    let view_class = if view_mode == ViewMode::DiffsOnly {
        "btn active"
    } else {
        "btn"
    };

    rsx! {
        div { class: "toolbar",
            span { class: "toolbar-title", "Döff" }
            span { class: "toolbar-sep" }
            button {
                class: "{view_class}",
                onclick: move |_| on_toggle_view.call(()),
                "{view_label}"
            }
            button { class: "{ws_class}", onclick: move |_| on_toggle_ws.call(()), "Ignore whitespace" }
            span { class: "toolbar-sep" }
            button { class: "btn", onclick: move |_| on_toggle_theme.call(()), "{theme_label}" }
        }
    }
}
