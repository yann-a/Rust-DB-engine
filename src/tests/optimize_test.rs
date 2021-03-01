use crate::optimize::*;
use crate::types::*;

#[test]
fn test_apply_projections_early() {
    //Available columns: idp,titre,responsable
    let expression = Box::new(Expression::Project(
        Box::new(Expression::Rename(
            Box::new(Expression::Load("project_spec/samples/projets.csv".to_string(), None)),
            vec![String::from("idp")],
            vec![String::from("truc")]
        )),
        vec!["truc".to_string(), "responsable".to_string()]
    ));

    // optimize this expression
    let optimizer = ChainOptimizer{optimizers: vec![
        Box::new(DetectLoadColumnsOptimizer{}),
        Box::new(ApplyProjectionsEarlyOptimizer{}),
    ]};
    let expression = optimizer.optimize(expression);

    match *expression {
        Expression::Rename(project_on, _, _) =>
            match *project_on {
                Expression::Project(_, mut fields) => {
                    fields.sort();
                    assert_eq!(fields, vec!["idp", "responsable"]);
                },
                _ => panic!("Expected a rename expression")
            },
        _ => panic!("Expected a rename expression")
    }
}