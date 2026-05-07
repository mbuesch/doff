use crate::diff::{DiffKind, DiffRow};
use dioxus::prelude::*;

#[component]
pub fn ArrowRow(
    row: DiffRow,
    idx: usize,
    is_block_start: bool,
    have_multiple_lines: bool,
    on_copy_l2r: EventHandler<usize>,
    on_copy_r2l: EventHandler<usize>,
    on_block_l2r: EventHandler<usize>,
    on_block_r2l: EventHandler<usize>,
) -> Element {
    let show_l2r = row.kind == DiffKind::LeftOnly || row.kind == DiffKind::Modified;
    let show_r2l = row.kind == DiffKind::RightOnly || row.kind == DiffKind::Modified;
    let show_block = is_block_start && have_multiple_lines && row.kind != DiffKind::Equal;
    let bid = row.block_id;

    rsx! {
        div { class: "cell-arrows",
            if show_block {
                button {
                    class: "arrow-btn block-arrow-btn",
                    title: "Copy block ←",
                    onclick: move |_| on_block_r2l.call(bid),
                    "«"
                }
            }
            if show_r2l {
                button {
                    class: "arrow-btn",
                    title: "Copy line ←",
                    onclick: move |_| on_copy_r2l.call(idx),
                    "←"
                }
            }
            if show_l2r {
                button {
                    class: "arrow-btn",
                    title: "Copy line →",
                    onclick: move |_| on_copy_l2r.call(idx),
                    "→"
                }
            }
            if show_block {
                button {
                    class: "arrow-btn block-arrow-btn",
                    title: "Copy block →",
                    onclick: move |_| on_block_l2r.call(bid),
                    "»"
                }
            }
        }
    }
}
