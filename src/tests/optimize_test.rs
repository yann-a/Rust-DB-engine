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

#[test]
fn test_push_down_selections() {
    let expression = Box::new(get_expression_from_str(
        r#"
        {"operation": "selection", "args": {
            "condition": {"comparator": "<", "attribute1": "truc", "attribute2": "truc"},
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
        Box::new(PushDownSelectionsOptimizer{}),
    ]};
    let expression = optimizer.optimize(expression);

    let expected = get_expression_from_str(
        r#"{
            "operation": "renaming",
            "args": {
                "old attributes": ["idp"],
                "new attributes": ["truc"],
                "object": {
                    "operation": "selection", 
                    "args": {
                        "condition": {"comparator": "<", "attribute1": "idp", "attribute2": "idp"},
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

#[test]
fn test_push_down_selections_with_products() {
    let expression = Box::new(get_expression_from_str(
        r#"
        {"operation": "selection", "args": {
            "condition": {"comparator": "=", "attribute1": "idp", "attribute2": "idp"},
            "object": {
                "operation": "product",
                "args": {
                    "object1": {
                        "operation": "load",
                        "args": { "filename": "project_spec/samples/projets.csv"}
                    },
                    "object2": {
                        "operation": "load",
                        "args": { "filename": "project_spec/samples/departements.csv"}
                    }
                }
            }
        }}
        "#
    ));

    // optimize this expression
    let optimizer = ChainOptimizer{optimizers: vec![
        Box::new(DetectLoadColumnsOptimizer{}),
        Box::new(PushDownSelectionsOptimizer{}),
    ]};
    let expression = optimizer.optimize(expression);

    let expected = get_expression_from_str(
        r#"
        {
            "operation": "product",
            "args": {
                "object2": {
                    "operation": "load",
                    "args": { "filename": "project_spec/samples/departements.csv"}
                },
                "object1": {
                    "operation": "selection", 
                    "args": {
                        "condition": {"comparator": "=", "attribute1": "idp", "attribute2": "idp"},
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

#[test]
fn test_read_select_project_rename_fold() {
    let expression = Box::new(get_expression_from_str(
        r#"
        {"operation": "selection", "args": {
            "condition": {"comparator": "=", "attribute1": "idp", "attribute2": "idp"},
            "object": {
                "operation": "load",
                "args": { "filename": "project_spec/samples/projets.csv"}
            }
        }}
        "#
    ));

    // optimize this expression
    let optimizer = ChainOptimizer{optimizers: vec![
        Box::new(DetectLoadColumnsOptimizer{}),
        Box::new(FoldComplexExpressionsOptimizer{}),
    ]};
    let expression = optimizer.optimize(expression);

    let expected = get_expression_from_str(
        r#"
        {
            "operation": "rspr",
            "args": {
                "filename": "project_spec/samples/projets.csv",
                "condition": {"comparator": "=", "attribute1": "idp", "attribute2": "idp"},
                "old attributes": ["titre", "idp", "responsable"],
                "new attributes": ["titre", "idp", "responsable"]
            }
        }
        "#
    );

    assert_eq!(*expression, expected);
}