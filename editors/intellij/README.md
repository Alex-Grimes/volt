# ⚡ Volt for IntelliJ IDEA & JetBrains IDEs

JetBrains IDE plugin for **Volt**: Codebase Hotspot & Cognitive Complexity Analyzer.

Compatible with **IntelliJ IDEA**, **PyCharm**, **RustRover**, **GoLand**, **WebStorm**, **CLion**, **PhpStorm**, and **Rider**.

---

## Features

- **⚡ Gutter Line Markers**: Visual lightning icons on file headers and individual function declarations showing voltage score, churn, and complexity on hover.
- **📊 Sidebar ToolWindow (`Volt Hotspots`)**: Activity panel with a ranked list of high-voltage files and expandable function-level hotspots with double-click source navigation.
- **⚡ Status Bar Indicator**: Displays current file's voltage rating (`⚡ Volt: 45.2 (HIGH)`).
- **🛠️ Tools Menu Actions**: `Tools > Volt > Scan Workspace Hotspots`, `Refresh`, and `Clear`.

---

## Requirements

The plugin communicates with the standalone `volt-core` backend binary. Build it from the repository root:

```bash
cargo build --release
```

The plugin automatically searches for the executable in:
1. Workspace `target/release/volt-core` or `target/debug/volt-core`
2. System `PATH` (`volt-core`)

---

## Building the Plugin (.zip)

To build the distributable IntelliJ plugin archive using Gradle:

```bash
cd editors/intellij
./gradlew buildPlugin
```

The built distribution archive will be located at:
`editors/intellij/build/distributions/volt-intellij-0.1.0.zip`

---

## Installation in JetBrains IDEs

1. Open your JetBrains IDE (IntelliJ, PyCharm, RustRover, etc.).
2. Navigate to **Settings / Preferences > Plugins**.
3. Click the ⚙️ gear icon and select **Install Plugin from Disk...**.
4. Select `volt-intellij-0.1.0.zip` and restart the IDE.
