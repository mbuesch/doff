use std::path::PathBuf;

pub async fn pick_file() -> Option<(PathBuf, String)> {
    let cwd = std::env::current_dir().ok();
    let path = tokio::task::spawn_blocking(move || {
        let mut dialog = rfd::FileDialog::new();
        if let Some(dir) = cwd {
            dialog = dialog.set_directory(dir);
        }
        dialog.pick_file()
    })
    .await
    .ok()
    .flatten()?;

    let content = tokio::fs::read_to_string(&path).await.ok()?;
    Some((path, content))
}

pub async fn save_file(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    tokio::fs::write(path, content).await
}
