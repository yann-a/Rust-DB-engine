use crate::optimize::*;
use crate::parser::*;

#[test]
fn test_apply_projections_early() {
    let expression = Box::new(get_expression_from_str(
        r#"
        {"operation": "projection", "args": {
            "attributes": ["truc", "responsable"],
            "object": {
                "operation": "renaming",
                "args": {
                    "old attributes": ["idp"],
                    "new attributes": ["truc"],
                    "object": {
                        "operation": "load",
                        "args": { "filename": "project_spec/samples/projets.csv"}
                    }
                }
            }
        }}
        "#
    ));

    // optimize this expression
    let optimizer = ChainOptimizer{optimizers: vec![
        Box::new(DetectLoadColumnsOptimizer{}),
        Box::new(ApplyProjectionsEarlyOptimizer{}),
    ]};
    let expression = optimizer.optimize(expression);

    let expected = get_expression_from_str(
        r#"{
            "operation": "renaming",
            "args": {
                "old attributes": ["idp"],
                "new attributes": ["truc"],
                "object": {
                    "operation": "projection", 
                    "args": {
                        "attributes": ["idp", "responsable"],
                        "object": {
                        "operation": "load",
                        "args": { "filename": "project_spec/samples/projets.csv"}
                    }
                }
            }
        }}
        "#
    );

    assert_eq!(*expression, expected);
}