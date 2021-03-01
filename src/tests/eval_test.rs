use crate::eval::*;
use crate::types::*;

#[test]
fn test_load() {
    let (columns, entries) = eval(Box::new(Expression::Load(String::from("project_spec/samples/projets.csv"), None)));

    let mut columns = columns.keys().into_iter().cloned().collect::<Vec<String>>();
    columns.sort();

    assert_eq!(columns, vec!["idp", "responsable", "titre"]);
    assert_eq!(entries[0].len(), 3);
}