use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::folder_formatter::file_tree::FileType as FT;
use crate::folder_formatter::json_formatting::format_paths;

#[derive(Serialize)]
pub struct FileNode {
    name: String,
    path: String,
    is_dir: bool,
    children: Option<Vec<FileNode>>,
    // indicates if there are more children not yet loaded (for "lazy load")
    has_more: Option<bool>,
}

fn dir_has_children(p: &PathBuf) -> bool {
    match fs::read_dir(p) {
        Ok(mut rd) => rd.next().is_some(),
        Err(_) => false,
    }
}

fn build_node(p: &PathBuf, depth: u32, max_depth: Option<u32>) -> Result<FileNode, std::io::Error> {
    let meta = fs::symlink_metadata(p)?;
    let is_dir = meta.file_type().is_dir();
    let name = p
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let path_str = p.to_string_lossy().to_string();

    if is_dir {
        if let Some(max) = max_depth {
            if depth >= max {
                // reached max depth: do not recurse, mark has_more (if the directory is not empty)
                let has_more = dir_has_children(p);
                return Ok(FileNode {
                    name,
                    path: path_str,
                    is_dir,
                    children: None,
                    has_more: Some(has_more),
                });
            } else {
                let children = read_children(p, depth + 1, max_depth)?;
                return Ok(FileNode {
                    name,
                    path: path_str,
                    is_dir,
                    children: Some(children),
                    has_more: Some(false),
                });
            }
        } else {
            // unlimited depth
            let children = read_children(p, depth + 1, max_depth)?;
            return Ok(FileNode {
                name,
                path: path_str,
                is_dir,
                children: Some(children),
                has_more: Some(false),
            });
        }
    } else {
        // file or link
        return Ok(FileNode {
            name,
            path: path_str,
            is_dir,
            children: None,
            has_more: Some(false),
        });
    }
}

fn read_children(
    dir: &PathBuf,
    depth: u32,
    max_depth: Option<u32>,
) -> Result<Vec<FileNode>, std::io::Error> {
    let mut items = Vec::new();
    let read = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => {
            // return empty array while path permission or other errors occur (avoid entire operation failure)
            return Ok(vec![]);
        }
    };

    for entry_res in read {
        if let Ok(entry) = entry_res {
            let path = entry.path();
            // Optional: skip hidden files or folder
            // if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
            //     if fname.starts_with('.') {
            //         continue;
            //     }
            // }
            match build_node(&path, depth, max_depth) {
                Ok(node) => items.push(node),
                Err(_) => continue, // 單個項目錯誤跳過
            }
        }
    }

    // 按資料夾優先、再按名稱排序
    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(items)
}


pub fn read_directory(path: String, max_depth: Option<u32>) -> Result<FileNode, String> {
    let root = PathBuf::from(&path);
    if !root.exists() {
        return Err(format!("Path not found: {}", path));
    }

    build_node(&root, 0, max_depth).map_err(|e| e.to_string())
}

fn map_file_type(ft: fs::FileType) -> FT {
    if ft.is_dir() {
        FT::Directory
    } else if ft.is_symlink() {
        FT::Link
    } else {
        FT::File
    }
}

/// 收集 root 下的平坦 (path, FileType) 列表（iterative, 使用 DirEntry.file_type() 儘量避免多次 stat）
pub fn collect_paths(root: &Path, max_depth: Option<u32>) -> Vec<(String, FT)> {
    let mut out = Vec::new();
    let mut stack: Vec<(PathBuf, u32)> = Vec::new();
    stack.push((root.to_path_buf(), 0));

    while let Some((dir, depth)) = stack.pop() {
        let rd = match fs::read_dir(&dir) {
            Ok(rd) => rd,
            Err(_) => continue,
        };
        for entry_res in rd {
            if let Ok(entry) = entry_res {
                let path = entry.path();
                // 優先用 DirEntry.file_type()，若失敗再 fallback
                let ft = entry
                    .file_type()
                    .or_else(|_| fs::symlink_metadata(&path).map(|m| m.file_type()))
                    .ok();

                if let Some(ft) = ft {
                    let mapped = map_file_type(ft);
                    // store path as string (relative or absolute as you prefer)
                    let path_str = path.to_string_lossy().into_owned();
                    out.push((path_str.clone(), mapped.clone()));

                    // 若為目錄且未達 max_depth，push 到 stack 以繼續掃描
                    if matches!(mapped, FT::Directory) {
                        if max_depth.map_or(true, |m| depth + 1 <= m) {
                            stack.push((path, depth + 1));
                        }
                    }
                }
            }
        }
    }

    out
}


pub fn read_directory_fast(path: String, max_depth: Option<u32>) -> Result<String, String> {
    let root = Path::new(&path);
    if !root.exists() {
        return Err(format!("Path not found: {}", path));
    }

    let children = collect_paths(root, max_depth);
    // format_paths 會用 FileTree::new 去建樹並 serialize
    Ok(format_paths(&path, children))
}
