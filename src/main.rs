use std::{
    env,
    fs::{self, File},
    path::Path,
};

use colored::Colorize;
use clipboard::{ClipboardContext, ClipboardProvider};

fn parse_tree_line(line: &str) -> Result<(usize, String, bool), &'static str> {
    let line = line.trim_end();
    if line.is_empty() {
        return Err("empty line");
    }

    // Hapus komentar
    let line = match line.find('#') {
        Some(i) => &line[..i],
        None => line,
    }.trim_end();

    if line.is_empty() {
        return Err("empty after comment");
    }

    // Coba ekstrak nama setelah "── " atau "──"
    let name_part = if let Some(i) = line.rfind("── ") {
        &line[i + 3..]
    } else if let Some(i) = line.rfind("──") {
        &line[i + 2..]
    } else {
        // Fallback: ambil bagian terakhir setelah whitespace
        line.split_whitespace().last().unwrap_or(line)
    };

    let name_part = name_part.trim();
    if name_part.is_empty() {
        return Err("no name found");
    }

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

    // Hitung indent: cari posisi awal nama di line asli
    let start_idx = line.find(name_part).unwrap_or(0);
    let prefix = &line[..start_idx];

    // Normalisasi prefix: ganti semua non-spasi jadi spasi
    let normalized: String = prefix
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { ' ' })
        .collect();

    let leading_spaces = normalized.chars().take_while(|&c| c == ' ').count();
    let indent = leading_spaces / 4;

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

    // Cek reserved names (Windows)
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

    // Cek karakter ilegal
    for c in r#"<>:"/\|?*"#.chars() {
        if name.contains(c) {
            return false;
        }
    }

    // Tidak boleh diakhiri spasi atau titik (Windows)
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
    // Look for lines that start with some non-space character followed by a space/tab
    let mut indented_lines = 0;
    for line in content.lines().skip(1) { // Starting from the second row
        let trimmed_start = line.trim_start();
        if !trimmed_start.is_empty() && line.len() > trimmed_start.len() {
            // If the line has a space at the beginning
            indented_lines += 1;
        }
    }

    // If at least 2 lines are indented, we consider it a tree
    indented_lines >= 2 && content.lines().count() >= 2
}

fn create_structure(lines: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut path_stack: Vec<String> = Vec::new();

    for line in lines {
        let parsed = parse_tree_line(line);
        if let Err(_) = parsed {
            continue; // skip invalid/empty lines
        }

        let (indent, name, is_dir) = parsed.unwrap();

        if path_stack.is_empty() {
            // Root
            fs::create_dir_all(&name)?;
            if is_dir {
                path_stack.push(name);
            }
            continue;
        }

        // Sesuaikan stack berdasarkan indent
        let indent = if indent >= path_stack.len() {
            path_stack.len() - 1
        } else {
            indent
        };
        path_stack.truncate(indent + 1);

        // let full_path: String = path_stack.iter().chain(Some(&name)).collect::<Vec<_>>().join("/");
        let full_path = path_stack.iter()
            .map(|s| s.as_str())
            .chain(std::iter::once(name.as_str()))
            .collect::<Vec<_>>()
            .join("/");


        if is_dir {
            fs::create_dir_all(&full_path)?;
            path_stack.push(name);
        } else {
            fs::create_dir_all(Path::new(&full_path).parent().unwrap())?;
            File::create(&full_path)?;
        }
    }

    Ok(())
}

fn read_input() -> Result<(Vec<String>, String), Box<dyn std::error::Error>> {
    if let Some(arg) = env::args().nth(1) {
        let content = std::fs::read_to_string(&arg)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Ok((lines, "file".to_string()))
    } else {

        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|_| "clipboard init failed")?;

        let content = ctx.get_contents()
            .map_err(|_| "clipboard read failed")?;

        if content.trim().is_empty() {
            return Err("clipboard is empty".into());
        }

        if !content.contains("──") && !content.contains("├") && !content.contains("└") {
            return Err("not a tree structure".into());
        }

        // ✅ check whether it looks like a tree
        if !looks_like_tree(&content) {
            return Err("clipboard is not a tree-structure".into());
        }

        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Ok((lines, "clipboard".to_string()))
    }
}

fn is_valid_structure(lines: &[String]) -> bool {
    lines.iter().any(|line| parse_tree_line(line).is_ok())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (lines, source) = read_input()?;

    if !is_valid_structure(&lines) {
        eprintln!("❌ Input is empty or invalid.");
        std::process::exit(1);
    }

    println!("Read from {} ({} lines)", source, lines.len());
    println!("✅ Creating structure...");

    if let Err(e) = create_structure(&lines) {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }

    println!("✅ Done!");
    Ok(())
}