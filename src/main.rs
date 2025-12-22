// File: src\main.rs
// Author: Hadi Cahyadi <cumulus13@gmail.com>
// Date: 2025-12-13
// Description: Create Directory Structures from Tree-like Text
// License: MIT

use std::{
    env,
    fs::{self, File},
    path::Path,
};

use clap_version_flag::colorful_version;

use clipboard::{ClipboardContext, ClipboardProvider};

fn parse_tree_line(line: &str) -> Result<(usize, String, bool), &'static str> {
    let line = line.trim_end();
    if line.is_empty() {
        return Err("empty line");
    }

    // Delete comment - FIXED: proper multi-byte character detection
    let line = {
        let mut result = line;
        for (i, c) in line.char_indices() {
            if c == '#' || c == '✅' || c == '←' {
                result = &line[..i];
                break;
            }
        }
        result.trim_end()
    };

    if line.is_empty() {
        return Err("empty after comment");
    }

    // Extract the name by searching for the complete tree marker pattern
    // Pattern: "├── " atau "└── " (branch/corner + 2 horizontal + space)
    let name_part = if let Some(pos) = line.find("├── ") {
        &line[pos + "├── ".len()..]
    } else if let Some(pos) = line.find("└── ") {
        &line[pos + "└── ".len()..]
    } else {
        // Fallback for root or other formats
        line.split_whitespace().last().unwrap_or(line)
    };

    let name_part = name_part.trim();
    if name_part.is_empty() {
        return Err("no name found");
    }

    // Remove emoji icons (📄, 📁, etc) from the beginning
    let name_part = name_part
        .trim_start_matches(|c: char| {
            c == '📄' || c == '📁' || c == '📂' || c.is_whitespace()
        })
        .trim();

    let is_dir = name_part.ends_with('/');
    let mut name = if is_dir {
        name_part[..name_part.len() - 1].trim().to_string()
    } else {
        name_part.to_string()
    };

    name = name.trim().to_string();
    if name.is_empty() || !is_valid_filename(&name) {
        return Err("invalid file name");
    }

    // Calculate indent dynamically: count CHARACTERS (not bytes) before name
    // Look for where the name starts in character count form
    let chars_before_name = line.chars()
        .take_while(|c| !name_part.starts_with(&c.to_string()))
        .count();
    
    // Every 4 characters = 1 indent level
    let indent = chars_before_name / 4;

    Ok((indent, name, is_dir))
}

fn is_valid_filename(name: &str) -> bool {
    if name.is_empty() || name.len() > 255 {
        return false;
    }
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Check reserved names (Windows)
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

    // Illegal character check
    for c in r#"<>:"/\|?*"#.chars() {
        if name.contains(c) {
            return false;
        }
    }

    // Cannot end with a space or period (Windows)
    if trimmed.ends_with(' ') || trimmed.ends_with('.') {
        return false;
    }

    true
}

fn looks_like_tree(content: &str) -> bool {
    let tree_markers = ["├", "└", "─", "│", "┬", "┼"];

    // If it has at least one Unicode character tree, OK
    if tree_markers.iter().any(|m| content.contains(m)) {
        return content.lines().count() >= 2;
    }

    // Try indentation/space based tree structure detection
    let mut indented_lines = 0;
    for line in content.lines().skip(1) {
        let trimmed_start = line.trim_start();
        if !trimmed_start.is_empty() && line.len() > trimmed_start.len() {
            indented_lines += 1;
        }
    }

    indented_lines >= 2 && content.lines().count() >= 2
}

fn create_structure(lines: &[String], debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut path_stack: Vec<String> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let parsed = parse_tree_line(line);
        if let Err(_) = parsed {
            continue;
        }

        let (indent, name, is_dir) = parsed.unwrap();

        if debug {
            println!("[DEBUG] Line {}: indent={}, name='{}', is_dir={}", idx, indent, name, is_dir);
            println!("[DEBUG] Stack before: {:?}", path_stack);
        }

        // Split name by '&' to handle multiple files
        let names: Vec<String> = name
            .split('&')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if path_stack.is_empty() {
            // Root
            for n in &names {
                if is_dir {
                    fs::create_dir_all(n)?;
                    if debug {
                        println!("📁 Root: {}", n);
                    }
                } else {
                    File::create(n)?;
                    if debug {
                        println!("📄 Root file: {}", n);
                    }
                }
            }
            // Push FIRST name to stack for directory hierarchy tracking
            if is_dir && !names.is_empty() {
                path_stack.push(names[0].clone());
            }
            continue;
        }

        // Adjust stack based on indent
        // indent=1 means child of root (stack should have 1 item = root)
        // indent=2 means child of level 1 (stack should have 2 items)
        if indent > path_stack.len() {
            // Indent too deep, stay at current level
            if debug {
                eprintln!("⚠️ Warning: indent {} > stack size {}", indent, path_stack.len());
            }
        } else {
            path_stack.truncate(indent);
        }

        if debug {
            println!("[DEBUG] Stack after truncate: {:?}", path_stack);
        }

        // Create all files from the split
        for n in &names {
            let full_path = path_stack.iter()
                .map(|s| s.as_str())
                .chain(std::iter::once(n.as_str()))
                .collect::<Vec<_>>()
                .join("/");

            if is_dir {
                fs::create_dir_all(&full_path)?;
                if debug {
                    println!("📁 {}", full_path);
                }
            } else {
                fs::create_dir_all(Path::new(&full_path).parent().unwrap())?;
                File::create(&full_path)?;
                if debug {
                    println!("📄 {}", full_path);
                }
            }
        }

        // Push ONLY FIRST name to stack for directory tracking
        if is_dir && !names.is_empty() {
            path_stack.push(names[0].clone());
        }

        if debug {
            println!("[DEBUG] Stack after: {:?}\n", path_stack);
        }
    }

    Ok(())
}

fn read_input() -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Check for file argument (skip --debug if present)
    let file_arg = if args.len() > 1 {
        if args[1] == "--debug" && args.len() > 2 {
            Some(&args[2])
        } else if args[1] != "--debug" {
            Some(&args[1])
        } else {
            None
        }
    } else {
        None
    };

    if let Some(file_path) = file_arg {
        let content = std::fs::read_to_string(file_path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        return Ok((lines, "file".to_string()));
    }

    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|_| "clipboard init failed")?;

    let content = ctx.get_contents()
        .map_err(|_| "clipboard read failed")?;

    if content.trim().is_empty() {
        return Err("clipboard is empty".into());
    }

    if !looks_like_tree(&content) {
        return Err("clipboard is not a tree-structure".into());
    }

    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    Ok((lines, "clipboard".to_string()))
}

fn is_valid_structure(lines: &[String]) -> bool {
    lines.iter().any(|line| parse_tree_line(line).is_ok())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let debug = args.contains(&"--debug".to_string());
    let version = args.contains(&"--version".to_string()) || args.contains(&"-V".to_string());
    let version_str = colorful_version!();
    
    let (lines, source) = read_input()?;

    if !is_valid_structure(&lines) {
        eprintln!("❌ Input is empty or invalid.");
        std::process::exit(1);
    }

    println!("📋 Read from {} ({} lines)", source, lines.len());
    
    if debug {
        println!("🪲 Debug mode enabled\n");
    }

    if version {
        println!("{}", version_str);
    }
    
    println!("✅ Creating structure...\n");

    if let Err(e) = create_structure(&lines, debug) {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }

    println!("\n✅ Done!");
    Ok(())
}