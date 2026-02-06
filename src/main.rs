mod analyzer;
use analyzer::CodeAnalyzer;

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
    let mut rust_analyzer = CodeAnalyzer::new(tree_sitter_rust::LANGUAGE.into());
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

    let mut final_scores: Vec<(String, f64)> = Vec::new();

    for (path_str, churn) in voltage_map {
        let path = Path::new(&path_str);

        if path.exists() && path.extension().map_or(false, |ext| ext == "rs") {
            if let Ok(content) = fs::read_to_string(path) {
                let complexity = rust_analyzer.score(&content);

                let score = (churn as f64) * (complexity as f64).sqrt();
                final_scores.push((path_str, score));
            }
        }
    }

    //final_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let final_scores_structs: Vec<VoltResult> = final_scores
        .into_iter()
        .map(|(path, score)| VoltResult {
            file_path: path,
            score,
            churn: 0,
            complexity: 0,
        })
        .collect();

    let output = serde_json::to_string(&final_scores_structs)?;
    println!("{}", output);

    //println!("{:<40} | {:<10}", "File Path", "Volt Score");
    //println!("{:-<55}", "");
    //for (path, score) in final_scores.iter().take(10) {
    //  println!("{:<40} | {:<10.2}", path, score);
    //}

    Ok(())
}
