# `mks` – Create Directory Structures from Tree-like Text

`mks` is a lightweight, cross-platform CLI tool that reads a directory structure in **tree format** (from clipboard or file) and automatically creates the corresponding folders and files.

Perfect for quickly scaffolding projects from shared diagrams, documentation, or terminal output.

---

## 🎦 Demo

<div align="center">
  <a href="https://youtu.be/1BrUiQWi47Y">
    <img src="https://i9.ytimg.com/vi_webp/1BrUiQWi47Y/mqdefault.webp?sqp=CLyA8MkG&rs=AOn4CLDK-YxnbjSj94eEnGR5YpQSSVmK1g" alt="How to use mks - tree2 -pt" style="width:100%;">
  </a>
  <br>
  <a href="https://youtu.be/1BrUiQWi47Y">Demo</a>
</div>


## ✅ Features

- **Input from clipboard** or **text file**
- Supports **Unix-style `tree` output** (with `├──`, `└──`, `│`)
- Also supports **simple indented format** using **spaces or tabs**
- **Windows-safe**: validates file/folder names (blocks `CON`, `NUL`, invalid chars)
- Creates **empty files** and **nested directories** as specified
- Fast, dependency-light, and compiles to a single executable

---

## 🚀 Quick Start

### 0. Install
```bash
cargo install mks
```

#### ✅ Format A: Simple Indent (Recommended)
Use **spaces or tabs** for nesting (no special symbols needed):

```text
my-app/
    package.json
    src/
        index.js
        utils/
            helper.js
    public/
        style.css
```

#### ✅ Format B: `tree` Output (Unix-style)
Copy directly from `tree` command in **Git Bash**, **WSL**, or **Linux/macOS**:

```text
my-app/
├── package.json
├── src/
│   ├── index.js
│   └── utils/
│       └── helper.js
└── public/
    └── style.css
```

---

### 3. Run `mks`

#### From a file:
```bash
mks structure.txt
```

#### From clipboard:
```bash
# Copy your tree text, then run:
mks
```

✅ Output:
```
Read from file (7 lines)
✅ Creating structure...
✅ Done!
```

---

## 📁 Output Example

Given this input:
```text
blog/
    posts/
        first.md
    config.yaml
```

`mks` will create:
```
blog/
├── config.yaml
└── posts/
    └── first.md
```

All files are **empty** (0 bytes) — ideal for scaffolding.

---

## ⚠️ Limitations & Notes

- **Windows reserved names** (`CON`, `PRN`, `AUX`, `NUL`, `COM1`, `LPT1`, etc.) are **blocked**.
- Filenames cannot contain: `< > : " / \ | ? *`
- Filenames cannot end with space or dot (`.`)
- Maximum filename length: 255 characters
- On **Linux**, ensure `xclip` or `xsel` is installed for clipboard support:
  ```bash
  sudo apt install xclip    # Debian/Ubuntu
  ```

---

## 🔒 Safety First

`mks` **never overwrites** existing files.  
If a file or folder already exists, it is **skipped silently** (no error).

To start fresh, run `mks` in an **empty directory**.

---

## 🛠️ Build Your Own

```bash
cargo build --release  # As per your preference for executable name
cp target/release/mks /usr/local/bin
```

The tool uses the **MIT License** — free to use, modify, and distribute.

---

## 💡 Pro Tips

- Use **Git Bash** on Windows to generate valid `tree` output:
  ```bash
  tree my-project
  ```
- Prefer **space-indented format** if sharing across teams — it’s more portable.
- Combine with templates: generate structure → fill files later.

---

## 🙌 Author

[**Hadi Cahyadi**](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)


---

> “Scaffold fast, code faster.” — `mks`