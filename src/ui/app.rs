use crate::{
    diff::{
        compute_diff, copy_block_left_to_right, copy_block_right_to_left, copy_row_left_to_right,
        copy_row_right_to_left,
    },
    io::{pick_file, save_file},
    types::{FileContent, InitialFiles, Side, Theme, ViewMode},
    ui::{
        close_dialog::CloseDialog, diff_viewer::DiffViewer, panel_header::PanelHeader,
        toolbar::Toolbar,
    },
};
use dioxus::{
    desktop::{WindowCloseBehaviour, use_wry_event_handler, window},
    prelude::*,
};

const CSS: &str = include_str!("style.css");
const EDIT_BRIDGE_JS: &str = include_str!("edit_bridge.js");

async fn save_dirty_file(file: Signal<Option<FileContent>>, mut dirty: Signal<bool>) {
    if let Some(f) = file.read().as_ref()
        && let Some(path) = &f.path
        && let Err(e) = save_file(path, &f.content).await
    {
        eprintln!("Error saving file {}: {e}", path.display());
    } else {
        dirty.set(false);
    }
}

fn dismiss_close_dialog(mut show_dialog: Signal<bool>, and_close: bool) {
    show_dialog.set(false);
    let d = window();
    d.set_close_behavior(WindowCloseBehaviour::WindowCloses);
    if and_close {
        d.close();
    }
}

/// Replace a single line (1-indexed) in `content`, preserving any trailing newline.
fn patch_line(content: &mut String, line_num: usize, new_text: &str) {
    let tail = if content.ends_with('\n') { "\n" } else { "" };
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    if line_num >= 1 && line_num <= lines.len() {
        lines[line_num - 1] = new_text.to_string();
    }
    *content = format!("{}{}", lines.join("\n"), tail);
}

#[component]
pub fn App() -> Element {
    let mut left_file: Signal<Option<FileContent>> = use_signal(|| None);
    let mut right_file: Signal<Option<FileContent>> = use_signal(|| None);

    let mut theme = use_signal(Theme::default);
    let mut view_mode = use_signal(ViewMode::default);
    let mut ignore_ws = use_signal(|| false);
    let mut cursor_side: Signal<Side> = use_signal(|| Side::Left);
    let mut left_dirty: Signal<bool> = use_signal(|| false);
    let mut right_dirty: Signal<bool> = use_signal(|| false);
    let mut show_close_dialog: Signal<bool> = use_signal(|| false);

    // Load files specified on the command line.
    let initial_files = use_context::<InitialFiles>();
    use_hook(|| {
        let init = initial_files.clone();
        spawn(async move {
            if let Some(path) = init.left
                && let Ok(content) = tokio::fs::read_to_string(&path).await
            {
                left_file.set(Some(FileContent::new(Some(path), content)));
            }
            if let Some(path) = init.right
                && let Ok(content) = tokio::fs::read_to_string(&path).await
            {
                right_file.set(Some(FileContent::new(Some(path), content)));
            }
        });
    });

    // JavaScript bridge: forwards .cell-input edits to Rust via dioxus.send().
    // Runs once on first render; the spawned task runs for the lifetime of the app.
    use_hook(|| {
        let mut ev = document::eval(EDIT_BRIDGE_JS);
        let mut lf = left_file;
        let mut rf = right_file;
        let mut ld = left_dirty;
        let mut rd = right_dirty;
        spawn(async move {
            loop {
                let Ok(s) = ev.recv::<String>().await else {
                    break;
                };
                // Protocol: "side\x01lineNum\x01text"
                let parts: Vec<&str> = s.splitn(3, '\x01').collect();
                if parts.len() < 3 {
                    continue;
                }
                let side = parts[0];
                let ln_s = parts[1];
                let text = parts[2].to_string();
                let Ok(ln) = ln_s.parse::<usize>() else {
                    continue;
                };
                if ln == 0 {
                    continue;
                }
                match side {
                    "left" => {
                        if let Some(f) = lf.write().as_mut() {
                            patch_line(&mut f.content, ln, &text);
                            ld.set(true);
                        }
                    }
                    "right" => {
                        if let Some(f) = rf.write().as_mut() {
                            patch_line(&mut f.content, ln, &text);
                            rd.set(true);
                        }
                    }
                    _ => {}
                }
            }
        });
    });

    let diff_rows = use_memo(move || {
        let l = left_file.read();
        let r = right_file.read();
        if l.is_none() && r.is_none() {
            return vec![];
        }
        let lc = l.as_ref().map(|f| f.content.as_str()).unwrap_or("");
        let rc = r.as_ref().map(|f| f.content.as_str()).unwrap_or("");
        compute_diff(lc, rc, *ignore_ws.read())
    });

    {
        let desktop = window();
        use_wry_event_handler(move |event, _| {
            use dioxus::desktop::tao::event::{Event, WindowEvent};
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } if (*left_dirty.peek() || *right_dirty.peek()) => {
                    desktop.set_close_behavior(WindowCloseBehaviour::WindowHides);
                    show_close_dialog.set(true);
                    // dioxus hides the window after this handler returns (WindowHides
                    // behaviour).  Re-show it so the close dialog is visible.
                    let d = desktop.clone();
                    spawn(async move {
                        d.set_visible(true);
                    });
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    let half_width = desktop.inner_size().width as f64 / 2.0;
                    cursor_side.set(if position.x > half_width {
                        Side::Right
                    } else {
                        Side::Left
                    });
                }
                Event::WindowEvent {
                    event: WindowEvent::DroppedFile(path),
                    ..
                } => {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let fc = FileContent::new(Some(path.clone()), content);
                        if *cursor_side.peek() == Side::Left {
                            left_file.set(Some(fc));
                            left_dirty.set(false);
                        } else {
                            right_file.set(Some(fc));
                            right_dirty.set(false);
                        }
                    }
                }
                _ => {}
            }
        });
    }

    let theme_class = match *theme.read() {
        Theme::Light => "app theme-light",
        Theme::Dark => "app theme-dark",
    };
    let rows = diff_rows.read().clone();

    rsx! {
        style { "{CSS}" }
        div { class: "{theme_class}",
            Toolbar {
                theme: theme.read().clone(),
                view_mode: view_mode.read().clone(),
                ignore_ws: *ignore_ws.read(),
                on_toggle_theme: move |_| {
                    theme
                        .set(if *theme.read() == Theme::Light { Theme::Dark } else { Theme::Light });
                },
                on_toggle_view: move |_| {
                    view_mode
                        .set(
                            if *view_mode.read() == ViewMode::Full {
                                ViewMode::DiffsOnly
                            } else {
                                ViewMode::Full
                            },
                        );
                },
                on_toggle_ws: move |_| {
                    let v = *ignore_ws.peek();
                    ignore_ws.set(!v);
                },
            }
            div { class: "panel-headers",
                PanelHeader {
                    file: left_file.read().clone(),
                    side: Side::Left,
                    dirty: *left_dirty.read(),
                    on_open: move |_| {
                        spawn(async move {
                            if let Some((path, content)) = pick_file().await {
                                left_file.set(Some(FileContent::new(Some(path), content)));
                                left_dirty.set(false);
                            }
                        });
                    },
                    on_save: move |_| {
                        spawn(async move {
                            save_dirty_file(left_file, left_dirty).await;
                        });
                    },
                }
                div { class: "panel-header-sep" }
                PanelHeader {
                    file: right_file.read().clone(),
                    side: Side::Right,
                    dirty: *right_dirty.read(),
                    on_open: move |_| {
                        spawn(async move {
                            if let Some((path, content)) = pick_file().await {
                                right_file.set(Some(FileContent::new(Some(path), content)));
                                right_dirty.set(false);
                            }
                        });
                    },
                    on_save: move |_| {
                        spawn(async move {
                            save_dirty_file(right_file, right_dirty).await;
                        });
                    },
                }
            }
            div { class: "diff-outer",
                if left_file.read().is_none() && right_file.read().is_none() {
                    div { class: "empty-panel", "Open or drag files onto either side to compare them." }
                } else {
                    DiffViewer {
                        rows: rows.clone(),
                        view_mode: view_mode.read().clone(),
                        on_copy_row_l2r: move |idx: usize| {
                            let rf = right_file.read().clone();
                            if let Some(rf) = rf
                                && let Some(row) = diff_rows.read().get(idx) {
                                let lines = copy_row_left_to_right(row, &rf.lines());
                                let tail = if rf.content.ends_with('\n') { "\n" } else { "" };
                                right_file
                                    .set(
                                        Some(
                                            FileContent::new(rf.path, format!("{}{tail}", lines.join("\n"))),
                                        ),
                                    );
                                right_dirty.set(true);
                            }
                        },
                        on_copy_row_r2l: move |idx: usize| {
                            let lf = left_file.read().clone();
                            if let Some(lf) = lf
                                && let Some(row) = diff_rows.read().get(idx) {
                                let lines = copy_row_right_to_left(row, &lf.lines());
                                let tail = if lf.content.ends_with('\n') { "\n" } else { "" };
                                left_file
                                    .set(
                                        Some(
                                            FileContent::new(lf.path, format!("{}{tail}", lines.join("\n"))),
                                        ),
                                    );
                                left_dirty.set(true);
                            }
                        },
                        on_copy_block_l2r: move |bid: usize| {
                            let rf = right_file.read().clone();
                            if let Some(rf) = rf {
                                let snap = diff_rows.read().clone();
                                let lines = copy_block_left_to_right(bid, &snap, &rf.lines());
                                let tail = if rf.content.ends_with('\n') { "\n" } else { "" };
                                right_file
                                    .set(
                                        Some(
                                            FileContent::new(rf.path, format!("{}{tail}", lines.join("\n"))),
                                        ),
                                    );
                                right_dirty.set(true);
                            }
                        },
                        on_copy_block_r2l: move |bid: usize| {
                            let lf = left_file.read().clone();
                            if let Some(lf) = lf {
                                let snap = diff_rows.read().clone();
                                let lines = copy_block_right_to_left(bid, &snap, &lf.lines());
                                let tail = if lf.content.ends_with('\n') { "\n" } else { "" };
                                left_file
                                    .set(
                                        Some(
                                            FileContent::new(lf.path, format!("{}{tail}", lines.join("\n"))),
                                        ),
                                    );
                                left_dirty.set(true);
                            }
                        },
                    }
                }
            }
            if *show_close_dialog.read() {
                CloseDialog {
                    left_dirty: *left_dirty.read(),
                    right_dirty: *right_dirty.read(),
                    on_save_and_close: move |_| {
                        if *left_dirty.read() {
                            spawn(async move {
                                save_dirty_file(left_file, left_dirty).await;
                            });
                        }
                        if *right_dirty.read() {
                            spawn(async move {
                                save_dirty_file(right_file, right_dirty).await;
                            });
                        }
                        if !*left_dirty.read() && !*right_dirty.read() {
                            dismiss_close_dialog(show_close_dialog, true);
                        }
                    },
                    on_close: move |_| dismiss_close_dialog(show_close_dialog, true),
                    on_cancel: move |_| dismiss_close_dialog(show_close_dialog, false),
                }
            }
        }
    }
}
