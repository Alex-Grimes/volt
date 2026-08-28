mod analyzer;
mod cli;

pub use analyzer::{CodeAnalyzer, FunctionHotspot, SupportedLanguage, calculate_voltage_score};
use clap::Parser;
use cli::{Cli, OutputFormat, format_table};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use std::{collections::HashMap, error::Error, fs, path::Path, process};

use git2::{DiffOptions, Repository};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct VoltResult {
    pub file_path: String,
    pub score: f64,
    pub churn: usize,
    pub complexity: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<FunctionHotspot>,
}

fn analyze_repository(cli: &Cli) -> Result<Vec<VoltResult>, Box<dyn Error>> {
    let repo = Repository::discover(&cli.path).map_err(|e| {
        format!(
            "Failed to find git repository at '{}': {}",
            cli.path.display(),
            e
        )
    })?;

    let repo_root = repo.workdir().unwrap_or(Path::new("."));

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut voltage_map: HashMap<String, usize> = HashMap::new();

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let current_tree = commit.tree()?;

        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut opts = DiffOptions::new();
        let diff =
            repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&current_tree), Some(&mut opts))?;

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                    *voltage_map.entry(path.to_string()).or_insert(0) += 1;
                }
                true
            },
            None,
            None,
            None,
        )?;
    }

    let mut final_scores: Vec<VoltResult> = voltage_map
        .into_par_iter()
        .filter_map(|(path_str, churn)| {
            let path = repo_root.join(&path_str);
            if !path.exists() {
                return None;
            }

            let lang = SupportedLanguage::from_path(&path)?;

            if !cli.include_ext.is_empty() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !cli.include_ext.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    return None;
                }
            }

            let content = fs::read_to_string(&path).ok()?;
            let mut analyzer = CodeAnalyzer::new(lang);
            let (complexity, functions) = analyzer.analyze(&content, churn);
            let score = calculate_voltage_score(churn, complexity);

            if cli.min_score.is_some_and(|min_score| score < min_score) {
                return None;
            }

            Some(VoltResult {
                file_path: path_str,
                score,
                churn,
                complexity,
                functions,
            })
        })
        .collect();

    final_scores.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(top) = cli.top {
        final_scores.truncate(top);
    }

    Ok(final_scores)
}

fn main() {
    let cli = Cli::parse();

    match analyze_repository(&cli) {
        Ok(results) => match cli.resolved_format() {
            OutputFormat::Json => {
                let output =
                    serde_json::to_string(&results).expect("Failed to serialize results to JSON");
                println!("{}", output);
            }
            OutputFormat::Table => {
                let output = format_table(&results, cli.functions);
                print!("{}", output);
            }
        },
        Err(err) => {
            eprintln!("⚡ Volt Error: {}", err);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volt_result_json_serialization() {
        let results = vec![
            VoltResult {
                file_path: "src/main.rs".to_string(),
                score: 42.5,
                churn: 5,
                complexity: 72,
                functions: vec![FunctionHotspot {
                    name: "main".to_string(),
                    line: 112,
                    end_line: 132,
                    complexity: 10,
                    score: 15.8,
                }],
            },
            VoltResult {
                file_path: "src/lib.rs".to_string(),
                score: 10.0,
                churn: 2,
                complexity: 25,
                functions: vec![],
            },
        ];

        let json = serde_json::to_string(&results).expect("Serialization failed");
        let deserialized: Vec<VoltResult> =
            serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(results, deserialized);
        assert!(json.contains("\"file_path\":\"src/main.rs\""));
        assert!(json.contains("\"score\":42.5"));
        assert!(json.contains("\"churn\":5"));
        assert!(json.contains("\"complexity\":72"));
        assert!(json.contains("\"name\":\"main\""));
    }

    #[test]
    fn test_volt_result_sorting() {
        let mut results = vec![
            VoltResult {
                file_path: "low.rs".to_string(),
                score: 5.0,
                churn: 1,
                complexity: 25,
                functions: vec![],
            },
            VoltResult {
                file_path: "high.rs".to_string(),
                score: 100.0,
                churn: 10,
                complexity: 100,
                functions: vec![],
            },
            VoltResult {
                file_path: "medium.rs".to_string(),
                score: 50.0,
                churn: 5,
                complexity: 100,
                functions: vec![],
            },
        ];

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        assert_eq!(results[0].file_path, "high.rs");
        assert_eq!(results[1].file_path, "medium.rs");
        assert_eq!(results[2].file_path, "low.rs");
    }

    #[test]
    fn test_top_and_min_score_filtering() {
        let mut results = vec![
            VoltResult {
                file_path: "a.rs".to_string(),
                score: 100.0,
                churn: 10,
                complexity: 100,
                functions: vec![],
            },
            VoltResult {
                file_path: "b.rs".to_string(),
                score: 50.0,
                churn: 5,
                complexity: 100,
                functions: vec![],
            },
            VoltResult {
                file_path: "c.rs".to_string(),
                score: 10.0,
                churn: 2,
                complexity: 25,
                functions: vec![],
            },
        ];

        // Filter min_score >= 40.0
        results.retain(|r| r.score >= 40.0);
        assert_eq!(results.len(), 2);

        // Truncate to top 1
        results.truncate(1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].file_path, "a.rs");
    }
}
