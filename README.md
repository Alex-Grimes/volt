# ⚡ volt.nvim & volt-core

> High-voltage codebase hotspot analyzer for Neovim and the terminal.  
> Identifies high-risk, complex, and frequently modified code before bugs strike.

```
       ⚡ Volt: High Voltage Hotspot Report ⚡
──────────────────────────────────────────────────────────────────────────────────
File Path                                          │ Volt Score │    Churn │ Complexity
──────────────────────────────────────────────────┼────────────┼──────────┼───────────
src/analyzer.rs                                    │     127.00 │        4 │       1008
  ↳ L271-L344 fn extract_function_name             │      73.86 │        - │        341
  ↳ L223-L269 fn score_node                        │      37.95 │        - │         90
src/main.rs                                        │      97.86 │        6 │        266
  ↳ L24-L113 fn analyze_repository                 │      82.92 │        - │        191
──────────────────────────────────────────────────────────────────────────────────
```

---

## ⚡ The Voltage Formula

Code hotspots emerge at the intersection of **frequent change** and **high cognitive complexity**. Volt measures this using a depth-weighted Tree-sitter AST traversal coupled with Git revision history:

$$\text{Voltage Score} = \text{Git Churn} \times \sqrt{\text{AST Complexity}}$$

- **Git Churn**: Number of revisions/commits touching a file across repository history.
- **AST Complexity**: Depth-weighted nesting complexity for control flow (`if`, `match`, loops, exception handlers, comprehensions) and function declarations.
- **Function-Level Granularity**: Extracts precise line ranges and isolated complexity scores for individual functions and methods.

---

## 🚀 Features

- **Multi-Language Support**: Tree-sitter parsers for **Rust**, **Go**, **Java**, **Python**, **JavaScript**, **TypeScript**, and **TSX**.
- **Function-Level Hotspots**: Identifies high-voltage functions and decorates their declaration lines in Neovim.
- **Blazing Fast**: Parallel AST parsing across all CPU cores using `rayon`.
- **Neovim Native**:
  - Floating report window with syntax highlighting and split navigation.
  - Built-in **`Snacks.picker`** and **Telescope** picker integrations.
  - Live buffer signs (`⚡`) and inline virtual text.
  - In-memory result cache with automatic buffer decoration.
- **Standalone CLI (`volt-core`)**: Output JSON for editor integration or formatted ASCII tables for terminal audits.

---

## 📦 Installation (Neovim)

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
  "Alex-Grimes/volt",
  build = "cargo build --release",
  cmd = { "VoltScan", "VoltSummary", "VoltPicker", "VoltSnacks", "VoltRefresh", "VoltClear" },
  opts = {
    picker = "auto", -- "auto" | "snacks" | "telescope"
    auto_annotate = true,
  },
}
```

### Using [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  "Alex-Grimes/volt",
  run = "cargo build --release",
  config = function()
    require("volt").setup({})
  end,
}
```

---

## 💻 Installation (VS Code)

1. Build the backend binary:
   ```bash
   cargo build --release
   ```
2. Build or install the extension from [`editors/vscode`](./editors/vscode):
   ```bash
   cd editors/vscode
   npm install && npm run compile
   npx @vscode/vsce package
   ```
3. In VS Code, run **Extensions > ... > Install from VSIX...** and select the generated `.vsix`.

---

## ⚙️ Configuration

Volt works out of the box with zero configuration, but can be customized:

```lua
require("volt").setup({
  -- Custom path to volt-core binary (optional, auto-detected from build/PATH)
  binary_path = nil,

  -- Sign column icons
  signs = {
    enable = true,
    icon = "⚡",
    priority = 10,
  },

  -- Inline virtual text annotations
  virtual_text = {
    enable = true,
    prefix = " ⚡ Voltage: ",
  },

  -- Function-level line annotations
  functions = {
    enable = true,
    min_complexity = 5, -- Only annotate functions with complexity >= threshold
    show_name = true,
  },

  -- Highlight thresholds
  thresholds = {
    high = 50,    -- DiagnosticError
    medium = 20,  -- DiagnosticWarn
    low = 5,      -- DiagnosticInfo
  },

  -- Auto-decorate buffers on open
  auto_annotate = true,

  -- Picker: "auto" (detects snacks -> telescope -> summary window), "snacks", or "telescope"
  picker = "auto",

  -- Floating window border style
  border = "rounded",
})
```

---

## ⌨️ Neovim Commands & Navigation

| Command | Description |
|---|---|
| `:VoltScan` | Scan project in the background and decorate open buffers |
| `:VoltSummary` | Open the interactive floating hotspot report |
| `:VoltPicker` | Open configured/detected picker (`snacks.picker` or `telescope`) |
| `:VoltSnacks` | Open `snacks.picker` with hotspot results |
| `:VoltTelescope` | Open `telescope` picker with hotspot results |
| `:VoltRefresh` | Force a fresh project rescan and update annotations |
| `:VoltClear` | Clear all gutter signs, virtual text, and cached data |

### Floating Summary Keymaps

Inside `:VoltSummary`:
- `<CR>`: Jump to file in active window
- `v` / `<C-v>`: Open file in vertical split
- `s` / `<C-s>`: Open file in horizontal split
- `t` / `<C-t>`: Open file in new tab
- `r`: Rescan and refresh summary
- `q` / `<Esc>`: Close window

---

## 🖥️ Standalone CLI Usage (`volt-core`)

You can run `volt-core` directly from your terminal or CI pipeline:

```bash
# Formatted table output
volt-core --table

# Include function-level hotspot breakdown
volt-core --table --functions

# Limit to top 10 files with a minimum voltage score of 20
volt-core --table -n 10 -m 20

# Filter by file extension
volt-core --table --include-ext rs,go,ts

# Analyze a specific repository path
volt-core /path/to/repo --table

# Emit raw JSON
volt-core /path/to/repo
```

---

## 🌐 Supported Languages

| Language | Extensions | Key AST Constructs Scored |
|---|---|---|
| **Rust** | `.rs` | `if`, `match`, `while`, `for`, `loop`, `try`, closures, `fn` |
| **Go** | `.go` | `if`, `for`, `switch`, `select`, `type_switch`, goroutines, methods, `func` |
| **Java** | `.java` | `if`, `while`, `for`, enhanced `for`, `do`, `switch`, `catch`, lambdas, methods |
| **Python** | `.py`, `.pyi` | `if`, `elif`, `for`, `while`, `match`/`case`, `except`, comprehensions, `def` |
| **JavaScript** | `.js`, `.jsx`, `.mjs`, `.cjs` | `if`, `for`, `for-in`, `for-of`, `while`, `switch`, `catch`, ternary, arrow fns, methods |
| **TypeScript** | `.ts`, `.mts`, `.cts` | Interfaces, generics, `if`, `for`, `switch`, `catch`, ternary, arrow fns, methods |
| **TSX** | `.tsx` | TSX elements, JSX expressions, nested conditionals, arrow components |

---

## 🛠️ Development & Testing

```bash
# Run all 30 unit & CLI integration tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Check code formatting
cargo fmt --check

# Test Neovim plugin in headless mode
nvim --headless -u NONE -i NONE -c "set rtp+=." -c "runtime plugin/volt.lua" -c "lua require('volt').setup({})" -c "q"
```

---

## 📄 License

MIT License.
