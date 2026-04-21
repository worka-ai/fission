use fission_core::AppState;
use fission_macros::Action;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

// --- State ---

#[derive(Debug, Clone)]
pub struct EditorState {
    // File tree
    pub root_path: PathBuf,
    pub tree_expanded: HashSet<String>,
    pub tree_selected: Option<String>,

    // Open files / tabs
    pub open_tabs: Vec<TabInfo>,
    pub active_tab: usize,

    // Editor content (path -> content)
    pub file_contents: HashMap<String, FileBuffer>,

    // UI state
    pub show_command_palette: bool,
    pub command_query: String,
    pub show_find_replace: bool,
    pub find_query: String,
    pub replace_query: String,
    pub sidebar_visible: bool,
    pub sidebar_section: SidebarSection,
    pub terminal_visible: bool,
    pub terminal_lines: Vec<String>,
    pub status_message: Option<String>,

    // Split
    pub sidebar_width: f32,
    pub terminal_height: f32,

    // LSP
    pub diagnostics: HashMap<String, Vec<Diagnostic>>,
    pub completions: Vec<CompletionItem>,
    pub show_completions: bool,
    pub hover_info: Option<String>,

    // Terminal input
    pub terminal_input: String,

    // Search
    pub search_query: String,
    pub search_results: Vec<SearchResult>,

    // Git
    pub git_status_lines: Vec<GitStatusEntry>,

    // Bottom panel tabs
    pub bottom_panel_tab: BottomPanelTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottomPanelTab {
    Terminal,
    Problems,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            tree_expanded: HashSet::new(),
            tree_selected: None,
            open_tabs: Vec::new(),
            active_tab: 0,
            file_contents: HashMap::new(),
            show_command_palette: false,
            command_query: String::new(),
            show_find_replace: false,
            find_query: String::new(),
            replace_query: String::new(),
            sidebar_visible: true,
            sidebar_section: SidebarSection::Explorer,
            terminal_visible: true,
            terminal_lines: vec!["Fission Editor v0.1.0".into(), "Ready.".into()],
            status_message: None,
            sidebar_width: 240.0,
            terminal_height: 150.0,
            diagnostics: HashMap::new(),
            completions: Vec::new(),
            show_completions: false,
            hover_info: None,
            terminal_input: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            git_status_lines: Vec::new(),
            bottom_panel_tab: BottomPanelTab::Terminal,
        }
    }
}

impl AppState for EditorState {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub path: String,
    pub title: String,
    pub is_dirty: bool,
}

#[derive(Debug, Clone)]
pub struct FileBuffer {
    pub content: String,
    pub language: Language,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Toml,
    Markdown,
    Json,
    Plain,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Language::Rust,
            "toml" => Language::Toml,
            "md" => Language::Markdown,
            "json" => Language::Json,
            _ => Language::Plain,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Language::Rust => "Rust",
            Language::Toml => "TOML",
            Language::Markdown => "Markdown",
            Language::Json => "JSON",
            Language::Plain => "Plain Text",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SidebarSection {
    Explorer,
    Search,
    Git,
    Extensions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub line: usize,
    pub col: usize,
    pub severity: DiagSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String, // "function", "variable", "keyword", etc.
    pub detail: Option<String>,
}

// --- Actions ---

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OpenFile(pub String);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CloseTab(pub usize);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SelectTab(pub usize);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ToggleTreeNode(pub String);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SelectTreeNode(pub String);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct UpdateFileContent(pub String);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ToggleCommandPalette;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct UpdateCommandQuery(pub String);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ToggleSidebar;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ToggleTerminal;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SetSidebarSection(pub SidebarSection);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SaveFile;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Noop;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SaveAllFiles;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct UpdateTerminalInput(pub String);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SubmitTerminalCommand;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct UpdateSearchQuery(pub String);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ExecuteSearch;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SelectCompletion(pub usize);

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DismissCompletions;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RefreshGitStatus;

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NavigateDiagnostic {
    pub path: String,
    pub line: usize,
}

// --- Additional types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub line: usize,
    pub col: usize,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusEntry {
    pub status: String,
    pub path: String,
}

// --- Helpers ---

impl EditorState {
    pub fn open_file(&mut self, path: String) {
        // Check if already open
        if let Some(idx) = self.open_tabs.iter().position(|t| t.path == path) {
            self.active_tab = idx;
            return;
        }

        // Read file
        let content = std::fs::read_to_string(&path).unwrap_or_else(|_| String::new());
        let ext = Path::new(&path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let lang = Language::from_extension(ext);
        let title = Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&path)
            .to_string();

        self.file_contents.insert(
            path.clone(),
            FileBuffer {
                content,
                language: lang,
                cursor_line: 0,
                cursor_col: 0,
            },
        );

        self.open_tabs.push(TabInfo {
            path: path.clone(),
            title,
            is_dirty: false,
        });
        self.active_tab = self.open_tabs.len() - 1;
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.open_tabs.len() {
            let tab = self.open_tabs.remove(idx);
            self.file_contents.remove(&tab.path);
            if self.active_tab >= self.open_tabs.len() && self.active_tab > 0 {
                self.active_tab -= 1;
            }
        }
    }

    pub fn active_buffer(&self) -> Option<(&TabInfo, &FileBuffer)> {
        self.open_tabs.get(self.active_tab).and_then(|tab| {
            self.file_contents.get(&tab.path).map(|buf| (tab, buf))
        })
    }

    pub fn active_buffer_mut(&mut self) -> Option<(&TabInfo, &mut FileBuffer)> {
        let tab = self.open_tabs.get(self.active_tab)?;
        let path = tab.path.clone();
        let buf = self.file_contents.get_mut(&path)?;
        let tab = &self.open_tabs[self.active_tab];
        Some((tab, buf))
    }

    pub fn save_active_file(&mut self) {
        if let Some(tab) = self.open_tabs.get(self.active_tab) {
            let path = tab.path.clone();
            if let Some(buf) = self.file_contents.get(&path) {
                if std::fs::write(&path, &buf.content).is_ok() {
                    if let Some(tab) = self.open_tabs.get_mut(self.active_tab) {
                        tab.is_dirty = false;
                    }
                    self.status_message = Some(format!("Saved {}", path));
                } else {
                    self.status_message = Some(format!("Failed to save {}", path));
                }
            }
        }
    }

    pub fn save_all_files(&mut self) {
        for i in 0..self.open_tabs.len() {
            if self.open_tabs[i].is_dirty {
                let path = self.open_tabs[i].path.clone();
                if let Some(buf) = self.file_contents.get(&path) {
                    if std::fs::write(&path, &buf.content).is_ok() {
                        self.open_tabs[i].is_dirty = false;
                    }
                }
            }
        }
        self.status_message = Some("All files saved".into());
    }

    pub fn run_terminal_command(&mut self) {
        let cmd = self.terminal_input.trim().to_string();
        if cmd.is_empty() { return; }
        self.terminal_lines.push(format!("$ {}", cmd));
        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .current_dir(&self.root_path)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                for line in stdout.lines() {
                    self.terminal_lines.push(line.to_string());
                }
                for line in stderr.lines() {
                    self.terminal_lines.push(format!("ERR: {}", line));
                }
            }
            Err(e) => {
                self.terminal_lines.push(format!("Error: {}", e));
            }
        }
        self.terminal_input.clear();
    }

    pub fn run_search(&mut self) {
        let query = self.search_query.clone();
        if query.is_empty() {
            self.search_results.clear();
            return;
        }
        let mut results = Vec::new();
        // Search in open buffers first
        for (path, buf) in &self.file_contents {
            for (line_idx, line) in buf.content.lines().enumerate() {
                if let Some(col) = line.find(&query) {
                    results.push(SearchResult {
                        path: path.clone(),
                        line: line_idx + 1,
                        col,
                        context: line.trim().to_string(),
                    });
                }
            }
        }
        // Search files on disk
        search_files_recursive(&self.root_path, &query, &mut results, 0);
        self.search_results = results;
    }

    pub fn refresh_git_status(&mut self) {
        match std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.root_path)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                self.git_status_lines = stdout.lines().filter_map(|line| {
                    if line.len() >= 3 {
                        Some(GitStatusEntry {
                            status: line[..2].trim().to_string(),
                            path: line[3..].to_string(),
                        })
                    } else {
                        None
                    }
                }).collect();
            }
            Err(_) => {
                self.git_status_lines.clear();
            }
        }
    }
}

// --- File tree scanning ---

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileEntry>,
}

pub fn scan_directory(path: &Path, depth: usize) -> Vec<FileEntry> {
    if depth > 4 {
        return Vec::new();
    }
    let mut entries = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(path) else {
        return entries;
    };

    let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    items.sort_by(|a, b| {
        let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });

    for item in items {
        let name = item.file_name().to_string_lossy().to_string();
        // Skip hidden, target, node_modules
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }
        let item_path = item.path();
        let is_dir = item.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let path_str = item_path.to_string_lossy().to_string();

        let children = if is_dir {
            scan_directory(&item_path, depth + 1)
        } else {
            Vec::new()
        };

        entries.push(FileEntry {
            name,
            path: path_str,
            is_dir,
            children,
        });
    }
    entries
}

fn search_files_recursive(dir: &Path, query: &str, results: &mut Vec<SearchResult>, depth: usize) {
    if depth > 3 || results.len() > 100 { return; }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" || name == "node_modules" { continue; }
        let path = entry.path();
        if path.is_dir() {
            search_files_recursive(&path, query, results, depth + 1);
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "rs" | "toml" | "md" | "json" | "txt" | "yaml" | "yml") { continue; }
            if let Ok(content) = std::fs::read_to_string(&path) {
                for (line_idx, line) in content.lines().enumerate() {
                    if let Some(col) = line.find(query) {
                        results.push(SearchResult {
                            path: path.to_string_lossy().to_string(),
                            line: line_idx + 1,
                            col,
                            context: line.trim().to_string(),
                        });
                        if results.len() > 100 { return; }
                    }
                }
            }
        }
    }
}
