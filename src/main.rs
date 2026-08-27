mod analyzer;
use analyzer::{CodeAnalyzer, SupportedLanguage};

use serde::Serialize;

use std::{collections::HashMap, error::Error, fs, path::Path};

use git2::{DiffOptions, Repository};

#[derive(Serialize)]
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
                    let score = (churn as f64) * (complexity as f64).sqrt();

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
