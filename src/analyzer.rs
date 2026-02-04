use tree_sitter::{Parser, TreeCursor};

pub struct CodeAnalyzer {
    parser: Parser,
}

impl CodeAnalyzer {
    pub fn new(lang: tree_sitter::Language) -> Self {
        let mut parser = Parser::new();
        parser.set_language(&lang).expect("Error loading languange");
        Self { parser }
    }

    pub fn score(&mut self, source: &str) -> usize {
        let tree = self.parser.parse(source, None).unwrap();
        let mut cursor = tree.walk();
        self.traverse(&mut cursor)
    }

    fn traverse(&self, cursor: &mut TreeCursor) -> usize {
        let mut complexity = 0;
        let mut depth = 0;
        loop {
            let node = cursor.node();

            match node.kind() {
                "if_expression" | "while_expression" | "for_expression" | "match_arm"
                | "loop_expression" | "match_expression" => {
                    complexity += 1 + depth;
                }

                "function_item" => {
                    complexity += 1;
                }
                _ => {}
            }

            if cursor.goto_first_child() {
                depth += 1;
                continue;
            }

            if cursor.goto_next_sibling() {
                continue;
            }

            loop {
                if !cursor.goto_parent() {
                    return complexity;
                }
                depth -= 1;
                if cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_complexity() {
        let mut analyzer = CodeAnalyzer::new(tree_sitter_rust::LANGUAGE.into());
        let code = r#"
            fn complex_function(x: i32) {
                if x > 0 {
                    for i in 0..10 {
                        println!("{}", i);
                    }
                }
            }
        "#;
        let score = analyzer.score(code);
        assert!(
            score > 0,
            "Complexity should be greater than 0 for nested logic"
        );
        println!("Complexity score: {}", score);
    }
}
