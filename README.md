# fwalker-rs

A high-performance Rust library for file system traversal and formatting, designed for efficiently scanning directory structures and generating structured file tree representations.

## Features

- **Fast Scanning**: Iterative algorithm avoids deep recursion for better performance
- **Flexible Depth Control**: Support for configurable maximum scan depth
- **Multiple Output Formats**: Tree structure and JSON format output
- **Symbolic Link Support**: Proper handling of symbolic links
- **Automatic Sorting**: Directories first, then alphabetically by name
- **Performance Optimized**: Uses `DirEntry.file_type()` to minimize system calls

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
fwalker-rs = "0.1.0"
```

## Usage

### Basic Usage

```rust
use fwalker_rs::{read_directory, read_directory_fast};

// Method 1: Read directory and return structured data
let result = read_directory("/path/to/directory", Some(3))?;
println!("{}", serde_json::to_string_pretty(&result)?);

// Method 2: Fast read with direct JSON string output
let json = read_directory_fast("/path/to/directory", Some(3))?;
println!("{}", json);
```

### Collecting Path Lists

```rust
use fwalker_rs::collect_paths;
use std::path::Path;

let paths = collect_paths(Path::new("/path/to/directory"), Some(2));
for (path, file_type) in paths {
    println!("{:?}: {}", file_type, path);
}
```

### Unlimited Depth Scan

```rust
// Pass None to scan the entire directory tree (no depth limit)
let result = read_directory("/path/to/directory", None)?;
```

## API Documentation

### `read_directory`

```rust
pub fn read_directory<P: AsRef<Path>>(
    path: P, 
    max_depth: Option<u32>
) -> Result<FileNode, std::io::Error>
```

Recursively reads directory structure and returns a nested `FileNode` tree.

**Parameters:**
- `path`: The directory path to scan
- `max_depth`: Maximum scan depth, `None` for unlimited

**Returns:**
A `FileNode` containing the complete directory structure

### `read_directory_fast`

```rust
pub fn read_directory_fast<P: AsRef<Path>>(
    path: P, 
    max_depth: Option<u32>
) -> Result<String, std::io::Error>
```

Quickly scans directory and directly returns JSON formatted string. Better performance than `read_directory`.

### `collect_paths`

```rust
pub fn collect_paths(
    root: &Path, 
    max_depth: Option<u32>
) -> Vec<(String, FileType)>
```

Collects a flat list of all files and directories, returning tuples of path and file type.

### `FileNode` Structure

```rust
pub struct FileNode {
    pub name: String,        // File/directory name
    pub path: String,        // Full path
    pub is_dir: bool,        // Whether it's a directory
    pub children: Option<Vec<FileNode>>,  // Child nodes
    pub has_more: Option<bool>,           // Whether there are more unloaded children
}
```

## Example Output

```json
{
  "name": "project",
  "path": "/home/user/project",
  "is_dir": true,
  "children": [
    {
      "name": "src",
      "path": "/home/user/project/src",
      "is_dir": true,
      "children": [
        {
          "name": "main.rs",
          "path": "/home/user/project/src/main.rs",
          "is_dir": false,
          "children": null,
          "has_more": false
        }
      ],
      "has_more": false
    },
    {
      "name": "Cargo.toml",
      "path": "/home/user/project/Cargo.toml",
      "is_dir": false,
      "children": null,
      "has_more": false
    }
  ],
  "has_more": false
}
```

## Error Handling

- Returns `NotFound` error when path doesn't exist
- Permission errors for individual files/directories are skipped without affecting the overall scan
- Failed symbolic link reads fall back to file type marking

## Performance Considerations

- Uses `Slab` for efficient memory management
- Uses `IndexMap` to maintain insertion order
- Prioritizes `DirEntry.file_type()` to avoid extra system calls
- Iterative depth-first traversal avoids stack overflow

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Contributing

Issues and Pull Requests are welcome!

