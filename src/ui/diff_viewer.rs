use crate::{
    diff::{DiffKind, DiffRow, InlinePart, visible_ranges},
    types::{Side, ViewMode},
    ui::diff_viewer::arrow_row::ArrowRow,
};
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};

mod arrow_row;

#[derive(Clone)]
enum Item {
    Sep { from_ln: usize, to_ln: usize },
    Row(usize),
}

#[component]
pub fn DiffViewer(
    rows: Vec<DiffRow>,
    view_mode: ViewMode,
    on_copy_row_l2r: EventHandler<usize>,
    on_copy_row_r2l: EventHandler<usize>,
    on_copy_block_l2r: EventHandler<usize>,
    on_copy_block_r2l: EventHandler<usize>,
) -> Element {
    let items = build_items(&rows, &view_mode);
    let block_starts = first_row_of_each_block(&items, &rows);
    let block_counts = block_row_counts(&items, &rows);

    rsx! {
        div { class: "diff-scroll",
            div { class: "diff-table",
                for item in &items {
                    match item {
                        Item::Sep { from_ln, to_ln } => rsx! {
                            div { class: "group-separator", "@@ −{from_ln} ... +{to_ln} @@" }
                        },
                        Item::Row(idx) => {
                            let idx = *idx;
                            let row = rows[idx].clone();
                            let is_block_start = block_starts.contains(&idx);
                            let have_multiple_lines = block_counts
                                .get(&rows[idx].block_id)
                                .copied()
                                .unwrap_or(0) > 1;
                            let left_ln = row.left_line_num.map_or(String::new(), |n| n.to_string());
                            let right_ln = row.right_line_num.map_or(String::new(), |n| n.to_string());
                            let left_html = cell_html(
                                &row.left_inline,
                                row.left_text.as_deref().unwrap_or(""),
                                &row.kind,
                                Side::Left,
                            );
                            let right_html = cell_html(
                                &row.right_inline,
                                row.right_text.as_deref().unwrap_or(""),
                                &row.kind,
                                Side::Right,
                            );
                            let left_text_str = row.left_text.as_deref().unwrap_or("").to_string();
                            let right_text_str = row.right_text.as_deref().unwrap_or("").to_string();
                            let row_class = row_bg_class(&row.kind);
                            rsx! {
                                div { key: "{idx}", class: "diff-row {row_class}",
                                    div { class: "cell left",
                                        div { class: "cell-ln", "{left_ln}" }
                                        div { class: "cell-content",
                                            div { class: "cell-hl", dangerous_inner_html: "{left_html}" }
                                            if let Some(ln) = row.left_line_num {
                                                div {
                                                    class: "cell-input",
                                                    contenteditable: "plaintext-only",
                                                    "data-line-num": "{ln}",
                                                    "data-side": "left",
                                                    "data-content": "{left_text_str}",
                                                }
                                            }
                                        }
                                    }
                                    ArrowRow {
                                        row: row.clone(),
                                        idx,
                                        is_block_start,
                                        have_multiple_lines,
                                        on_copy_l2r: move |i| on_copy_row_l2r.call(i),
                                        on_copy_r2l: move |i| on_copy_row_r2l.call(i),
                                        on_block_l2r: move |b| on_copy_block_l2r.call(b),
                                        on_block_r2l: move |b| on_copy_block_r2l.call(b),
                                    }
                                    div { class: "cell right",
                                        div { class: "cell-ln", "{right_ln}" }
                                        div { class: "cell-content",
                                            div { class: "cell-hl", dangerous_inner_html: "{right_html}" }
                                            if let Some(ln) = row.right_line_num {
                                                div {
                                                    class: "cell-input",
                                                    contenteditable: "plaintext-only",
                                                    "data-line-num": "{ln}",
                                                    "data-side": "right",
                                                    "data-content": "{right_text_str}",
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cell_html(inline: &[InlinePart], plain_text: &str, kind: &DiffKind, side: Side) -> String {
    if inline.is_empty() {
        return html_escape(plain_text);
    }
    let hl_class = inline_hl_class(kind, side);
    inline
        .iter()
        .map(|p| {
            if p.highlighted {
                format!(
                    "<mark class=\"{}\">{}</mark>",
                    hl_class,
                    html_escape(&p.text)
                )
            } else {
                html_escape(&p.text)
            }
        })
        .collect()
}

fn row_bg_class(kind: &DiffKind) -> &'static str {
    match kind {
        DiffKind::Equal => "row-equal",
        DiffKind::LeftOnly => "row-left-only",
        DiffKind::RightOnly => "row-right-only",
        DiffKind::Modified => "row-modified",
    }
}

fn inline_hl_class(kind: &DiffKind, side: Side) -> &'static str {
    match (kind, side) {
        (DiffKind::Modified, _) => "hl-mod",
        (_, Side::Left) => "hl-left",
        (_, Side::Right) => "hl-right",
    }
}

fn build_items(rows: &[DiffRow], view_mode: &ViewMode) -> Vec<Item> {
    if *view_mode != ViewMode::DiffsOnly {
        return (0..rows.len()).map(Item::Row).collect();
    }

    let ranges = visible_ranges(rows, 3);
    let mut items: Vec<Item> = Vec::new();
    let mut prev_end: Option<usize> = None;

    for range in &ranges {
        if let Some(pe) = prev_end {
            let from_ln = rows
                .get(pe.saturating_sub(1))
                .and_then(|r| r.left_line_num)
                .unwrap_or(pe);
            let to_ln = rows
                .get(range.start)
                .and_then(|r| r.left_line_num)
                .unwrap_or(range.start + 1);
            items.push(Item::Sep { from_ln, to_ln });
        }
        for i in range.clone() {
            items.push(Item::Row(i));
        }
        prev_end = Some(range.end);
    }

    items
}

fn block_row_counts(items: &[Item], rows: &[DiffRow]) -> HashMap<usize, usize> {
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for item in items {
        if let Item::Row(idx) = item {
            let row = &rows[*idx];
            if row.kind != DiffKind::Equal {
                *counts.entry(row.block_id).or_insert(0) += 1;
            }
        }
    }
    counts
}

fn first_row_of_each_block(items: &[Item], rows: &[DiffRow]) -> HashSet<usize> {
    let mut seen: HashSet<usize> = HashSet::new();
    let mut starts: HashSet<usize> = HashSet::new();
    for item in items {
        if let Item::Row(idx) = item {
            let row = &rows[*idx];
            if row.kind != DiffKind::Equal && seen.insert(row.block_id) {
                starts.insert(*idx);
            }
        }
    }
    starts
}

/// Minimal HTML escaping for text injected via dangerous_inner_html.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
