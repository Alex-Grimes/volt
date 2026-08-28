use std::path::Path;
use tree_sitter::{Parser, TreeCursor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedLanguage {
    Rust,
    Go,
    Java,
    Python,
    JavaScript,
    TypeScript,
    Tsx,
}

impl SupportedLanguage {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "py" | "pyi" => Some(Self::Python),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "tsx" => Some(Self::Tsx),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    pub fn language(&self) -> tree_sitter::Language {
        match self {
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Go => tree_sitter_go::LANGUAGE.into(),
            Self::Java => tree_sitter_java::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        }
    }

    pub fn is_control_flow(&self, kind: &str) -> bool {
        match self {
            Self::Rust => matches!(
                kind,
                "if_expression"
                    | "while_expression"
                    | "for_expression"
                    | "match_arm"
                    | "loop_expression"
                    | "match_expression"
                    | "try_expression"
            ),
            Self::Go => matches!(
                kind,
                "if_statement"
                    | "for_statement"
                    | "expression_switch_statement"
                    | "type_switch_statement"
                    | "select_statement"
                    | "expression_case"
                    | "type_case"
                    | "communication_case"
            ),
            Self::Java => matches!(
                kind,
                "if_statement"
                    | "while_statement"
                    | "for_statement"
                    | "enhanced_for_statement"
                    | "do_statement"
                    | "switch_expression"
                    | "switch_block_statement_group"
                    | "switch_rule"
                    | "catch_clause"
                    | "ternary_expression"
            ),
            Self::Python => matches!(
                kind,
                "if_statement"
                    | "elif_clause"
                    | "while_statement"
                    | "for_statement"
                    | "match_statement"
                    | "case_clause"
                    | "except_clause"
                    | "conditional_expression"
                    | "list_comprehension"
                    | "dictionary_comprehension"
                    | "set_comprehension"
                    | "generator_expression"
            ),
            Self::JavaScript | Self::TypeScript | Self::Tsx => matches!(
                kind,
                "if_statement"
                    | "while_statement"
                    | "for_statement"
                    | "for_in_statement"
                    | "for_of_statement"
                    | "do_statement"
                    | "switch_statement"
                    | "switch_case"
                    | "switch_default"
                    | "catch_clause"
                    | "ternary_expression"
            ),
        }
    }

    pub fn is_function(&self, kind: &str) -> bool {
        match self {
            Self::Rust => matches!(kind, "function_item" | "closure_expression"),
            Self::Go => matches!(
                kind,
                "function_declaration" | "method_declaration" | "func_literal"
            ),
            Self::Java => matches!(
                kind,
                "method_declaration"
                    | "constructor_declaration"
                    | "compact_constructor_declaration"
                    | "lambda_expression"
            ),
            Self::Python => matches!(kind, "function_definition" | "lambda"),
            Self::JavaScript | Self::TypeScript | Self::Tsx => matches!(
                kind,
                "function_declaration"
                    | "function_expression"
                    | "arrow_function"
                    | "method_definition"
                    | "generator_function_declaration"
                    | "generator_function"
            ),
        }
    }
}

pub struct CodeAnalyzer {
    parser: Parser,
    lang: SupportedLanguage,
}

impl CodeAnalyzer {
    pub fn new(lang: SupportedLanguage) -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&lang.language())
            .expect("Error loading language");
        Self { parser, lang }
    }

    pub fn score(&mut self, source: &str) -> usize {
        let tree = match self.parser.parse(source, None) {
            Some(tree) => tree,
            None => return 0,
        };
        let mut cursor = tree.walk();
        self.traverse(&mut cursor)
    }

    fn traverse(&self, cursor: &mut TreeCursor) -> usize {
        let mut complexity = 0;
        let mut depth = 0;
        loop {
            let node = cursor.node();
            let kind = node.kind();

            if self.lang.is_control_flow(kind) {
                complexity += 1 + depth;
            } else if self.lang.is_function(kind) {
                complexity += 1;
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

pub fn calculate_voltage_score(churn: usize, complexity: usize) -> f64 {
    (churn as f64) * (complexity as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_complexity() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Rust);
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
        assert!(score > 0, "Rust complexity should be greater than 0");
    }

    #[test]
    fn test_go_complexity() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Go);
        let code = r#"
            package main
            func complexFunc(x int) {
                if x > 0 {
                    for i := 0; i < 10; i++ {
                        switch x {
                        case 1:
                            println("one")
                        default:
                            println("other")
                        }
                    }
                }
            }
        "#;
        let score = analyzer.score(code);
        assert!(score > 0, "Go complexity should be greater than 0");
    }

    #[test]
    fn test_java_complexity() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Java);
        let code = r#"
            class Calculator {
                public void compute(int x) {
                    if (x > 0) {
                        for (int i = 0; i < 10; i++) {
                            try {
                                System.out.println(i);
                            } catch (Exception e) {
                                e.printStackTrace();
                            }
                        }
                    }
                }
            }
        "#;
        let score = analyzer.score(code);
        assert!(score > 0, "Java complexity should be greater than 0");
    }

    #[test]
    fn test_python_complexity() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Python);
        let code = r#"
            def process_items(items):
                result = []
                for item in items:
                    if item > 0:
                        val = item * 2 if item % 2 == 0 else item
                        result.append(val)
                return result
        "#;
        let score = analyzer.score(code);
        assert!(score > 0, "Python complexity should be greater than 0");
    }

    #[test]
    fn test_javascript_complexity() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::JavaScript);
        let code = r#"
            function process(data) {
                if (!data) return;
                for (const item of data) {
                    if (item.active) {
                        console.log(item.name);
                    }
                }
            }
        "#;
        let score = analyzer.score(code);
        assert!(score > 0, "JavaScript complexity should be greater than 0");
    }

    #[test]
    fn test_typescript_complexity() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::TypeScript);
        let code = r#"
            function transform<T>(items: T[]): T[] {
                const results: T[] = [];
                for (let i = 0; i < items.length; i++) {
                    if (items[i]) {
                        results.push(items[i]);
                    }
                }
                return results;
            }
        "#;
        let score = analyzer.score(code);
        assert!(score > 0, "TypeScript complexity should be greater than 0");
    }

    #[test]
    fn test_tsx_complexity() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Tsx);
        let code = r#"
            export const Component = ({ items }: { items: string[] }) => {
                return (
                    <div>
                        {items.map(item => item ? <span>{item}</span> : null)}
                    </div>
                );
            };
        "#;
        let score = analyzer.score(code);
        assert!(score > 0, "TSX complexity should be greater than 0");
    }

    #[test]
    fn test_empty_and_comments_only() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Rust);
        assert_eq!(analyzer.score(""), 0);
        assert_eq!(
            analyzer.score("// just a comment\n/* block comment */"),
            0
        );

        let mut py_analyzer = CodeAnalyzer::new(SupportedLanguage::Python);
        assert_eq!(py_analyzer.score("# python comment\n"), 0);
    }

    #[test]
    fn test_syntax_error_resilience() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Rust);
        let broken_code = r#"
            fn valid_part_with_error(x: i32) {
                if x > 0 {
                    println!("valid if");
                }
                @@@ invalid syntax here ???
            }
        "#;
        let score = analyzer.score(broken_code);
        assert!(
            score > 0,
            "Analyzer should gracefully parse valid segments even with syntax errors"
        );
    }

    #[test]
    fn test_nesting_depth_increases_score() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Rust);

        let flat_code = r#"
            fn flat(a: bool, b: bool, c: bool) {
                if a { println!("a"); }
                if b { println!("b"); }
                if c { println!("c"); }
            }
        "#;

        let nested_code = r#"
            fn nested(a: bool, b: bool, c: bool) {
                if a {
                    if b {
                        if c {
                            println!("nested");
                        }
                    }
                }
            }
        "#;

        let flat_score = analyzer.score(flat_code);
        let nested_score = analyzer.score(nested_code);
        assert!(
            nested_score > flat_score,
            "Nested complexity ({}) must be strictly higher than flat complexity ({})",
            nested_score,
            flat_score
        );
    }

    #[test]
    fn test_rust_advanced_constructs() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Rust);
        let code = r#"
            fn process(val: Option<i32>) {
                let closure = |x: i32| x * 2;
                loop {
                    match val {
                        Some(1) => break,
                        Some(x) if x > 10 => {
                            while let Some(y) = Some(x) {
                                println!("{}", closure(y));
                            }
                        }
                        _ => {}
                    }
                }
            }
        "#;
        let score = analyzer.score(code);
        assert!(
            score > 5,
            "Advanced Rust constructs should yield a high score"
        );
    }

    #[test]
    fn test_go_advanced_constructs() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Go);
        let code = r#"
            package main
            func worker(ch1, ch2 chan int) {
                go func() {
                    for {
                        select {
                        case msg1 := <-ch1:
                            if msg1 > 0 {
                                println(msg1)
                            }
                        case msg2 := <-ch2:
                            println(msg2)
                        }
                    }
                }()
            }
        "#;
        let score = analyzer.score(code);
        assert!(
            score > 5,
            "Go channels, select, and goroutines should be scored"
        );
    }

    #[test]
    fn test_java_advanced_constructs() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Java);
        let code = r#"
            class Service {
                public void handle(List<String> items) {
                    for (String item : items) {
                        try {
                            int val = item.length() > 5 ? 1 : 0;
                            do {
                                val--;
                            } while (val > 0);
                        } catch (NullPointerException e) {
                            // handle
                        } catch (Exception e) {
                            // handle
                        }
                    }
                }
            }
        "#;
        let score = analyzer.score(code);
        assert!(
            score > 5,
            "Java enhanced for, do-while, ternary, multiple catch should be scored"
        );
    }

    #[test]
    fn test_python_advanced_constructs() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::Python);
        let code = r#"
            def complex_calc(data):
                squares = [x**2 for x in data if x > 0]
                evens = {k: v for k, v in data.items() if v % 2 == 0}
                match data:
                    case [1, 2]:
                        return True
                    case [x, y] if x > y:
                        return False
                    case _:
                        pass
                try:
                    res = 1 / len(squares)
                except ZeroDivisionError:
                    res = 0
                except Exception:
                    res = -1
                return res
        "#;
        let score = analyzer.score(code);
        assert!(
            score > 5,
            "Python comprehensions, match/case, and try/except should be scored"
        );
    }

    #[test]
    fn test_javascript_advanced_constructs() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::JavaScript);
        let code = r#"
            async function processAll(items) {
                for (const item of items) {
                    for (const key in item) {
                        switch (key) {
                            case "a":
                                console.log(item[key]);
                                break;
                            default:
                                break;
                        }
                    }
                }
                const fn = () => items.length > 0 ? true : false;
                try {
                    await Promise.all([]);
                } catch (e) {
                    console.error(e);
                }
            }
        "#;
        let score = analyzer.score(code);
        assert!(
            score > 5,
            "JS for-in, for-of, switch, ternary, try-catch, arrow fn should be scored"
        );
    }

    #[test]
    fn test_typescript_generics_and_types() {
        let mut analyzer = CodeAnalyzer::new(SupportedLanguage::TypeScript);
        let code = r#"
            interface Item<T> { data: T; }
            function processItem<T>(item: Item<T>): boolean {
                if (!item.data) {
                    return false;
                }
                for (let i = 0; i < 10; i++) {
                    if (i % 2 === 0) return true;
                }
                return false;
            }
        "#;
        let score = analyzer.score(code);
        assert!(
            score > 0,
            "TypeScript code with interfaces and generics should score properly"
        );
    }

    #[test]
    fn test_calculate_voltage_score() {
        assert_eq!(calculate_voltage_score(0, 100), 0.0);
        assert_eq!(calculate_voltage_score(10, 0), 0.0);
        assert_eq!(calculate_voltage_score(4, 9), 12.0); // 4 * sqrt(9) = 4 * 3 = 12.0
        assert_eq!(calculate_voltage_score(2, 16), 8.0); // 2 * sqrt(16) = 2 * 4 = 8.0
    }

    #[test]
    fn test_extension_detection() {
        assert_eq!(
            SupportedLanguage::from_path(Path::new("src/main.rs")),
            Some(SupportedLanguage::Rust)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("cmd/main.go")),
            Some(SupportedLanguage::Go)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("App.java")),
            Some(SupportedLanguage::Java)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("script.py")),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("index.js")),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("app.ts")),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("Component.tsx")),
            Some(SupportedLanguage::Tsx)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("src/component.test.tsx")),
            Some(SupportedLanguage::Tsx)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("scripts/bundle.min.js")),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("types/api.d.ts")),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("types/api.d.mts")),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("types/api.d.cts")),
            Some(SupportedLanguage::TypeScript)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("lib/module.mjs")),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("lib/module.cjs")),
            Some(SupportedLanguage::JavaScript)
        );
        assert_eq!(
            SupportedLanguage::from_path(Path::new("stubs.pyi")),
            Some(SupportedLanguage::Python)
        );
        assert_eq!(SupportedLanguage::from_path(Path::new("Dockerfile")), None);
        assert_eq!(SupportedLanguage::from_path(Path::new("Makefile")), None);
        assert_eq!(SupportedLanguage::from_path(Path::new("config.toml")), None);
        assert_eq!(SupportedLanguage::from_path(Path::new("data.json")), None);
        assert_eq!(SupportedLanguage::from_path(Path::new("README.md")), None);
    }
}
