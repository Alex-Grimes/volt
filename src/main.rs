mod analyzer;
use analyzer::CodeAnalyzer;

use std::collections::HashMap;

use git2::{DiffOptions, Repository};

fn main() -> Result<(), git2::Error> {
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

    let mut sorted_churn: Vec<_> = voltage_map.into_iter().collect();
    sorted_churn.sort_by(|a, b| b.1.cmp(&a.1));

    println!("--- Volt: High Voltage Files ---");
    for (path, count) in sorted_churn.iter().take(5) {
        println!("{:>4} commits | {}", count, path);
    }

    Ok(())
}
