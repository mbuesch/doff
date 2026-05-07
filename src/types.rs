use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct InitialFiles {
    pub left: Option<PathBuf>,
    pub right: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FileContent {
    pub path: Option<PathBuf>,
    pub content: String,
}

impl FileContent {
    pub fn new(path: Option<PathBuf>, content: String) -> Self {
        Self { path, content }
    }

    pub fn lines(&self) -> Vec<String> {
        self.content.lines().map(|l| l.to_string()).collect()
    }

    pub fn display_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("(no name)")
            .to_string()
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Full,
    DiffsOnly,
}
