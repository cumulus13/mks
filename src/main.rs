// File: src/main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2025-12-13
// Description: Create Directory Structures from Tree-like Text
// License: MIT

use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

use clap_version_flag::colorful_version;
use clipboard::{ClipboardContext, ClipboardProvider};

// ============================================================================
// STRUCTS & TYPES
// ============================================================================

#[derive(Debug, Clone)]
struct TreeNode {
    indent: usize,
    name: String,
    is_dir: bool,
    line_number: usize,
}

#[derive(Debug)]
enum ParseError {
    EmptyLine,
    EmptyAfterComment,
    NoNameFound,
    InvalidFilename,
}

// ============================================================================
// PATH UTILITIES
// ============================================================================

/// Extract base directory name from an absolute path
fn extract_base_name(path_str: &str) -> Option<String> {
    let path_str = path_str.trim_end_matches(&['/', '\\'][..]);
    let path = Path::new(path_str);
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.to_string())
}

/// Extract parent directory from absolute path
fn extract_parent_path(path_str: &str) -> Option<PathBuf> {
    let path_str = path_str.trim_end_matches(&['/', '\\'][..]);
    let path = Path::new(path_str);
    path.parent().map(|p| p.to_path_buf())
}

/// Check if a string looks like an absolute path
fn is_absolute_path(s: &str) -> bool {
    let s = s.trim();
    
    // Windows: C:\, D:\, etc.
    if s.len() >= 3 {
        let bytes = s.as_bytes();
        if bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
            return true;
        }
    }
    
    // UNC: \\server\share
    if s.starts_with("\\\\") || s.starts_with("//") {
        return true;
    }
    
    // Unix: /home, /usr
    if s.starts_with('/') {
        return true;
    }
    
    false
}

// ============================================================================
// PARSING FUNCTIONS
// ============================================================================

/// Remove emoji from start
fn strip_emoji_prefix(s: &str) -> &str {
    s.trim_start_matches(|c: char| {
        matches!(c, 
            '📄' | '📁' | '📂' | '📃' | '📋' | '📝' | 
            '🗂' | '🗃' | '📑' | '📊' | '📈' | '📉' |
            '✅' | '❌' | '⚠' | '🔴' | '🟢' | '🟡'
        ) || c.is_whitespace()
    }).trim()
}

/// Remove comments (# or emoji at end)
fn remove_comments(line: &str) -> &str {
    let mut result = line;
    for (i, c) in line.char_indices() {
        if c == '#' || c == '✅' || c == '❌' || c == '←' || c == '→' {
            result = &line[..i];
            break;
        }
    }
    result.trim_end()
}

/// Calculate indent by counting │ before name
fn calculate_indent(line: &str) -> usize {
    line.chars().filter(|&c| c == '│').count()
}

/// Extract name from tree line
fn extract_name_from_line(line: &str) -> Option<(String, Option<String>)> {
    // Try tree markers
    let markers = ["├── ", "└── ", "├─ ", "└─ ", "├─", "└─"];
    
    for marker in &markers {
        if let Some(pos) = line.find(marker) {
            let name = &line[pos + marker.len()..];
            return Some((name.trim().to_string(), None));
        }
    }
    
    // No marker - check if root line with absolute path
    let cleaned = line.trim();
    
    // Remove leading tree chars and emoji
    let cleaned = cleaned
        .trim_start_matches(|c: char| {
            matches!(c, '│' | '├' | '└' | '─' | '┬' | '┼' | ' ' | '\t')
        });
    
    let cleaned = strip_emoji_prefix(cleaned);
    
    if !cleaned.is_empty() {
        // Check if absolute path
        if is_absolute_path(cleaned) {
            // Return (base_name, full_path)
            if let Some(base) = extract_base_name(cleaned) {
                return Some((base, Some(cleaned.to_string())));
            }
        }
        
        // Otherwise return as-is (relative path)
        return Some((cleaned.to_string(), None));
    }
    
    None
}

/// Parse tree line into TreeNode
fn parse_tree_line(line: &str, line_number: usize) -> Result<(TreeNode, Option<String>), ParseError> {
    let line = line.trim_end();
    
    if line.is_empty() {
        return Err(ParseError::EmptyLine);
    }
    
    // Remove comments
    let line = remove_comments(line);
    if line.is_empty() {
        return Err(ParseError::EmptyAfterComment);
    }
    
    // Calculate indent FIRST (before modifying line)
    let indent = calculate_indent(line);
    
    // Extract name (returns (name, optional_full_path))
    let (mut name, full_path) = extract_name_from_line(line)
        .ok_or(ParseError::NoNameFound)?;
    
    // Remove emoji prefix again (might have some left)
    name = strip_emoji_prefix(&name).to_string();
    
    // Remove size info: (0.00 B), (1.2 KB), etc.
    if let Some(pos) = name.rfind(" (") {
        if name[pos..].contains("B)") || name[pos..].contains("KB)") || name[pos..].contains("MB)") {
            name = name[..pos].trim().to_string();
        }
    }
    
    // Check directory (ends with /)
    let is_dir = name.ends_with('/');
    if is_dir {
        name = name[..name.len() - 1].trim().to_string();
    }
    
    // Validate
    if name.is_empty() || !is_valid_filename(&name) {
        return Err(ParseError::InvalidFilename);
    }
    
    Ok((TreeNode {
        indent,
        name,
        is_dir,
        line_number,
    }, full_path))
}

// ============================================================================
// VALIDATION
// ============================================================================

fn is_valid_filename(name: &str) -> bool {
    if name.is_empty() || name.len() > 255 {
        return false;
    }
    
    let trimmed = name.trim();
    if trimmed.is_empty() || name.contains('\0') {
        return false;
    }
    
    if cfg!(target_os = "windows") {
        let upper = trimmed.to_uppercase();
        let base = upper.split('.').next().unwrap_or(&upper);
        let reserved = [
            "CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];
        if reserved.contains(&base) {
            return false;
        }
        
        for c in r#"<>:"/\|?*"#.chars() {
            if name.contains(c) {
                return false;
            }
        }
        
        if trimmed.ends_with(' ') || trimmed.ends_with('.') {
            return false;
        }
    } else {
        if name.contains('/') {
            return false;
        }
    }
    
    for c in name.chars() {
        if c.is_control() && c != '\t' {
            return false;
        }
    }
    
    true
}

fn looks_like_tree(content: &str) -> bool {
    let tree_markers = ["├", "└", "─", "│", "┬", "┼"];
    
    if tree_markers.iter().any(|m| content.contains(m)) {
        return content.lines().count() >= 2;
    }
    
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 2 {
        return false;
    }
    
    let mut indented_lines = 0;
    for line in lines.iter().skip(1) {
        let trimmed_start = line.trim_start();
        if !trimmed_start.is_empty() && line.len() > trimmed_start.len() {
            indented_lines += 1;
        }
    }
    
    indented_lines >= 2
}

// ============================================================================
// STRUCTURE CREATION
// ============================================================================

fn create_structure(
    nodes: &[TreeNode],
    base_path: PathBuf,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut path_stack: Vec<String> = Vec::new();
    
    if debug {
        println!("🎯 Base path: {}", base_path.display());
        println!("📊 Processing {} nodes\n", nodes.len());
    }
    
    for node in nodes {
        if debug {
            println!("[DEBUG] Line {}: indent={}, name='{}', is_dir={}", 
                     node.line_number, node.indent, node.name, node.is_dir);
            println!("[DEBUG] Stack before: {:?}", path_stack);
        }
        
        // Split by '&' for multiple files
        let names: Vec<String> = node.name
            .split('&')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        if names.is_empty() {
            continue;
        }
        
        // Root level (indent 0)
        if node.indent == 0 {
            path_stack.clear();
            
            for name in &names {
                let full_path = base_path.join(name);
                
                if node.is_dir {
                    fs::create_dir_all(&full_path)?;
                    if debug {
                        println!("📁 {}", full_path.display());
                    }
                } else {
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    File::create(&full_path)?;
                    if debug {
                        println!("📄 {}", full_path.display());
                    }
                }
            }
            
            // Push first name for hierarchy
            if node.is_dir && !names.is_empty() {
                path_stack.push(names[0].clone());
            }
            continue;
        }
        
        // Adjust stack for indent level
        if node.indent > path_stack.len() {
            if debug {
                eprintln!("⚠️  Line {}: indent {} > stack {} - adjusting", 
                         node.line_number, node.indent, path_stack.len());
            }
        } else if node.indent < path_stack.len() {
            path_stack.truncate(node.indent);
        }
        
        if debug {
            println!("[DEBUG] Stack after adjust: {:?}", path_stack);
        }
        
        // Create files/dirs
        for name in &names {
            let mut full_path = base_path.clone();
            for dir in &path_stack {
                full_path.push(dir);
            }
            full_path.push(name);
            
            if node.is_dir {
                fs::create_dir_all(&full_path)?;
                if debug {
                    println!("📁 {}", full_path.display());
                }
            } else {
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                File::create(&full_path)?;
                if debug {
                    println!("📄 {}", full_path.display());
                }
            }
        }
        
        // Push first name if directory
        if node.is_dir && !names.is_empty() {
            path_stack.push(names[0].clone());
        }
        
        if debug {
            println!("[DEBUG] Stack after: {:?}\n", path_stack);
        }
    }
    
    Ok(())
}

// ============================================================================
// INPUT
// ============================================================================

fn read_input(args: &[String]) -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
    let file_arg = args.iter()
        .skip(1)
        .find(|arg| !arg.starts_with("--") && !arg.starts_with('-'));
    
    if let Some(file_path) = file_arg {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read '{}': {}", file_path, e))?;
        
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        return Ok((lines, format!("file '{}'", file_path)));
    }
    
    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|_| "Failed to init clipboard")?;
    
    let content = ctx.get_contents()
        .map_err(|_| "Failed to read clipboard")?;
    
    if content.trim().is_empty() {
        return Err("Clipboard is empty".into());
    }
    
    if !looks_like_tree(&content) {
        return Err("Clipboard doesn't look like tree structure".into());
    }
    
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    Ok((lines, "clipboard".to_string()))
}

// ============================================================================
// MAIN
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let debug = args.iter().any(|arg| arg == "--debug" || arg == "-d");
    let version = args.iter().any(|arg| arg == "--version" || arg == "-V");
    let help = args.iter().any(|arg| arg == "--help" || arg == "-h");
    
    if help {
        print_help();
        return Ok(());
    }
    
    if version {
        println!("{}", colorful_version!());
        return Ok(());
    }
    
    let (lines, source) = read_input(&args)?;
    
    if lines.is_empty() {
        eprintln!("❌ Input is empty");
        std::process::exit(1);
    }
    
    println!("📋 Read from {} ({} lines)", source, lines.len());
    
    // Parse all lines
    let mut nodes: Vec<TreeNode> = Vec::new();
    let mut root_full_path: Option<String> = None;
    let mut parse_errors = 0;
    
    for (idx, line) in lines.iter().enumerate() {
        match parse_tree_line(line, idx + 1) {
            Ok((node, full_path)) => {
                // Save root full path if this is first node (line 1)
                if idx == 0 && full_path.is_some() {
                    root_full_path = full_path;
                }
                nodes.push(node);
            }
            Err(e) => {
                if debug {
                    println!("⚠️  Skipped line {}: {:?}", idx + 1, e);
                }
                parse_errors += 1;
            }
        }
    }
    
    if nodes.is_empty() {
        eprintln!("❌ No valid tree structure found");
        if parse_errors > 0 {
            eprintln!("   {} lines failed to parse", parse_errors);
        }
        std::process::exit(1);
    }
    
    if debug {
        println!("✅ Parsed {} nodes ({} errors)\n", nodes.len(), parse_errors);
    }
    
    // Determine base path
    let base_path = if let Some(full_path) = root_full_path {
        // Has absolute path - extract parent directory
        if let Some(parent) = extract_parent_path(&full_path) {
            println!("🌍 Detected absolute path: {}", full_path);
            println!("📁 Creating structure in: {}/\n", parent.display());
            
            // Create parent directory if needed
            if !parent.exists() {
                fs::create_dir_all(&parent)?;
                if debug {
                    println!("📁 Created parent: {}\n", parent.display());
                }
            }
            
            parent
        } else {
            // Can't extract parent - use current dir
            PathBuf::from(".")
        }
    } else {
        // No absolute path - use current directory
        if debug {
            println!("📁 Using current directory\n");
        }
        PathBuf::from(".")
    };
    
    println!("✅ Creating structure...\n");
    
    if let Err(e) = create_structure(&nodes, base_path, debug) {
        eprintln!("\n❌ Error: {}", e);
        std::process::exit(1);
    }
    
    println!("\n✅ Done! Successfully created {} items", nodes.len());
    Ok(())
}

fn print_help() {
    println!("Tree Structure Creator");
    println!();
    println!("USAGE:");
    println!("  mks [OPTIONS] [FILE]");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help       Show help");
    println!("  -V, --version    Show version");
    println!("  -d, --debug      Enable debug");
    println!();
    println!("ARGUMENTS:");
    println!("  [FILE]           Read from file (optional)");
    println!("                   Default: read from clipboard");
    println!();
    println!("EXAMPLES:");
    println!("  mks tree.txt");
    println!("  mks --debug");
}