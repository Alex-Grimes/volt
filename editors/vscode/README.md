# ⚡ Volt for Visual Studio Code

Visual Studio Code extension for **Volt**: Codebase Hotspot & Cognitive Complexity Analyzer.

---

## Features

- **⚡ Gutter Signs & Virtual Text**: Visual indicator on file headers and individual function declarations showing voltage score, churn, and complexity.
- **📊 Sidebar Hotspots Explorer**: Activity bar panel with a ranked list of high-voltage files and expandable function-level hotspots with direct jump navigation.
- **🔍 QuickPick Hotspots Navigator**: `Ctrl+Shift+P` / `Cmd+Shift+P` -> `Volt: Show Hotspots List` to fuzzy find and jump to hot files or functions.
- **⚡ Status Bar Indicator**: Displays current file's voltage score and severity level (`HIGH`, `MEDIUM`, `LOW`).

---

## Requirements

The extension requires the `volt-core` backend binary. Build it once from the repository root:

```bash
cargo build --release
```

The extension automatically searches for the binary in:
1. Custom path set in `volt.binaryPath`
2. Workspace `target/release/volt-core` or `target/debug/volt-core`
3. System `PATH` (`volt-core`)

---

## Extension Settings

| Setting | Default | Description |
|---|---|---|
| `volt.binaryPath` | `""` | Custom path to the `volt-core` executable |
| `volt.autoScan` | `true` | Automatically scan workspace on startup and document save |
| `volt.decorations.enable` | `true` | Show inline virtual text and gutter signs |
| `volt.decorations.showFunctions` | `true` | Show inline virtual text on function declarations |
| `volt.decorations.minFunctionComplexity` | `5` | Minimum complexity to annotate functions |
| `volt.thresholds.high` | `50` | Score threshold for High Voltage (Red) |
| `volt.thresholds.medium` | `20` | Score threshold for Medium Voltage (Yellow) |
| `volt.thresholds.low` | `5` | Score threshold for Low Voltage (Cyan) |

---

## Building & Packaging (.vsix)

```bash
cd editors/vscode
npm install
npm run compile
npx @vscode/vsce package
```

Install the generated `.vsix` in VS Code via **Extensions > ... > Install from VSIX...**
