mod analyzer;
use analyzer::{calculate_voltage_score, CodeAnalyzer, SupportedLanguage};

use serde::Serialize;

use std::{collections::HashMap, error::Error, fs, path::Path};

use git2::{DiffOptions, Repository};

#[derive(Serialize, serde::Deserialize, Debug, PartialEq)]
struct VoltResult {
    file_path: String,
    score: f64,
    churn: usize,
    complexity: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let repo = Repository::discover(".")?;
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

    let mut analyzers: HashMap<SupportedLanguage, CodeAnalyzer> = HashMap::new();
    let mut final_scores: Vec<VoltResult> = Vec::new();

    for (path_str, churn) in voltage_map {
        let path = Path::new(&path_str);

        if path.exists() {
            if let Some(lang) = SupportedLanguage::from_path(path) {
                if let Ok(content) = fs::read_to_string(path) {
                    let analyzer = analyzers
                        .entry(lang)
                        .or_insert_with(|| CodeAnalyzer::new(lang));
                    let complexity = analyzer.score(&content);
                    let score = calculate_voltage_score(churn, complexity);

                    final_scores.push(VoltResult {
                        file_path: path_str,
                        score,
                        churn,
                        complexity,
                    });
                }
            }
        }
    }

    final_scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let output = serde_json::to_string(&final_scores)?;
    println!("{}", output);

    Ok(())
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
            },
            VoltResult {
                file_path: "src/lib.rs".to_string(),
                score: 10.0,
                churn: 2,
                complexity: 25,
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
    }

    #[test]
    fn test_volt_result_sorting() {
        let mut results = vec![
            VoltResult {
                file_path: "low.rs".to_string(),
                score: 5.0,
                churn: 1,
                complexity: 25,
            },
            VoltResult {
                file_path: "high.rs".to_string(),
                score: 100.0,
                churn: 10,
                complexity: 100,
            },
            VoltResult {
                file_path: "medium.rs".to_string(),
                score: 50.0,
                churn: 5,
                complexity: 100,
            },
        ];

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        assert_eq!(results[0].file_path, "high.rs");
        assert_eq!(results[1].file_path, "medium.rs");
        assert_eq!(results[2].file_path, "low.rs");
    }
}
