use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Json,
    Table,
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "volt-core",
    about = "⚡ Codebase voltage analyzer: calculates churn × complexity hotspot scores",
    version
)]
pub struct Cli {
    /// Path to repository or sub-directory
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output format
    #[arg(short = 'f', long = "format", value_enum, default_value_t = OutputFormat::Json)]
    pub format: OutputFormat,

    /// Convenience flag for formatted table output
    #[arg(long = "table", conflicts_with = "format")]
    pub table: bool,

    /// Limit output to top N highest-voltage files
    #[arg(short = 'n', long = "top")]
    pub top: Option<usize>,

    /// Filter out files with voltage score below this threshold
    #[arg(short = 'm', long = "min-score")]
    pub min_score: Option<f64>,

    /// Only include files with specified extensions (e.g. rs, go, ts)
    #[arg(long = "include-ext", value_delimiter = ',')]
    pub include_ext: Vec<String>,

    /// Display function-level hotspot breakdown
    #[arg(long = "functions")]
    pub functions: bool,
}

impl Cli {
    pub fn resolved_format(&self) -> OutputFormat {
        if self.table {
            OutputFormat::Table
        } else {
            self.format
        }
    }
}

pub fn format_table(results: &[crate::VoltResult], show_functions: bool) -> String {
    if results.is_empty() {
        return "⚡ No matching voltage hotspots found.\n".to_string();
    }

    let mut out = String::new();
    out.push_str("⚡ Volt: High Voltage Hotspot Report\n");
    out.push_str(
        "──────────────────────────────────────────────────────────────────────────────────\n",
    );
    out.push_str(&format!(
        "{:<50} │ {:>10} │ {:>8} │ {:>10}\n",
        "File Path", "Volt Score", "Churn", "Complexity"
    ));
    out.push_str(
        "──────────────────────────────────────────────────┼────────────┼──────────┼───────────\n",
    );

    for item in results {
        let path_display = if item.file_path.len() > 48 {
            format!("...{}", &item.file_path[item.file_path.len() - 45..])
        } else {
            item.file_path.clone()
        };

        out.push_str(&format!(
            "{:<50} │ {:>10.2} │ {:>8} │ {:>10}\n",
            path_display, item.score, item.churn, item.complexity
        ));

        if show_functions && !item.functions.is_empty() {
            for func in &item.functions {
                let func_label = format!("  ↳ L{}-L{} fn {}", func.line, func.end_line, func.name);
                let func_display = if func_label.len() > 48 {
                    format!("{}...", &func_label[..45])
                } else {
                    func_label
                };
                out.push_str(&format!(
                    "{:<50} │ {:>10.2} │ {:>8} │ {:>10}\n",
                    func_display, func.score, "-", func.complexity
                ));
            }
        }
    }

    out.push_str(
        "──────────────────────────────────────────────────────────────────────────────────\n",
    );
    out.push_str(&format!("Total Analyzed Files: {}\n", results.len()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VoltResult;
    use crate::analyzer::FunctionHotspot;

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["volt-core"]);
        assert_eq!(cli.path, PathBuf::from("."));
        assert_eq!(cli.resolved_format(), OutputFormat::Json);
        assert_eq!(cli.top, None);
        assert_eq!(cli.min_score, None);
        assert!(cli.include_ext.is_empty());
        assert!(!cli.functions);
    }

    #[test]
    fn test_cli_flags() {
        let cli = Cli::parse_from([
            "volt-core",
            "../other-repo",
            "-n",
            "10",
            "-m",
            "25.5",
            "--table",
            "--include-ext",
            "rs,go,py",
            "--functions",
        ]);
        assert_eq!(cli.path, PathBuf::from("../other-repo"));
        assert_eq!(cli.resolved_format(), OutputFormat::Table);
        assert_eq!(cli.top, Some(10));
        assert_eq!(cli.min_score, Some(25.5));
        assert_eq!(cli.include_ext, vec!["rs", "go", "py"]);
        assert!(cli.functions);
    }

    #[test]
    fn test_format_table_empty() {
        let output = format_table(&[], false);
        assert!(output.contains("No matching voltage hotspots found"));
    }

    #[test]
    fn test_format_table_content() {
        let results = vec![
            VoltResult {
                file_path: "src/analyzer.rs".to_string(),
                score: 45.2,
                churn: 3,
                complexity: 227,
                functions: vec![FunctionHotspot {
                    name: "traverse".to_string(),
                    line: 20,
                    end_line: 58,
                    complexity: 50,
                    score: 21.2,
                }],
            },
            VoltResult {
                file_path: "a/very/long/nested/path/to/some/deeply/nested/component/file_name.tsx"
                    .to_string(),
                score: 12.0,
                churn: 2,
                complexity: 36,
                functions: vec![],
            },
        ];

        let output = format_table(&results, true);
        assert!(output.contains("Volt: High Voltage Hotspot Report"));
        assert!(output.contains("src/analyzer.rs"));
        assert!(output.contains("45.20"));
        assert!(output.contains("fn traverse"));
        assert!(output.contains("Total Analyzed Files: 2"));
    }
}
