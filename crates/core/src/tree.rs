use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileNodeType {
    Directory,
    File,
    Symlink,
    Junction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub allocated: u64,
    pub node_type: FileNodeType,
    pub children: Vec<FileNode>,
    pub item_count: u64,
    pub last_modified: Option<u64>,
}

impl FileNode {
    pub fn directory(name: &str, path: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            size: 0,
            allocated: 0,
            node_type: FileNodeType::Directory,
            children: Vec::new(),
            item_count: 1, // Count the directory itself
            last_modified: None,
        }
    }

    pub fn file(name: &str, path: &str, size: u64) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            size,
            allocated: size,
            node_type: FileNodeType::File,
            children: Vec::new(),
            item_count: 1,
            last_modified: None,
        }
    }

    pub fn total_size(&self) -> u64 {
        if self.node_type == FileNodeType::Directory {
            self.children.iter().map(|c| c.total_size()).sum()
        } else {
            self.size
        }
    }

    pub fn total_items(&self) -> u64 {
        if self.node_type == FileNodeType::Directory {
            self.children.iter().map(|c| c.total_items()).sum::<u64>() + 1
        } else {
            1
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub root: FileNode,
}
