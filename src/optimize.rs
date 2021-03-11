use crate::types::*;
use csv::Reader;
use std::collections::HashSet;
use std::collections::HashMap;

pub trait Optimizer {
    fn optimize(&self, expression: Box<Expression>) -> Box<Expression>;
}

pub struct ChainOptimizer { pub optimizers: Vec<Box<dyn Optimizer>> }
impl Optimizer for ChainOptimizer {
    fn optimize(&self, expression: Box<Expression>) -> Box<Expression> {
        let mut final_expression = expression;

        for optimizer in &self.optimizers {
            final_expression = optimizer.optimize(final_expression);
        }

        final_expression
    }
}

/**
 * Call the optimizer on the children of this expression.
 */
fn visit_children(optimizer: &dyn Optimizer, expression: Box<Expression>) -> Box<Expression> {
    Box::new(
        match *expression {
            Expression::Select(expression_from, condition) => Expression::Select(optimizer.optimize(expression_from), condition),
            Expression::Project(expression_from, columns) => Expression::Project(optimizer.optimize(expression_from), columns),
            Expression::Product(expr1, expr2) => Expression::Product(optimizer.optimize(expr1), optimizer.optimize(expr2)),
            Expression::Except(expr1, expr2) => Expression::Except(optimizer.optimize(expr1), optimizer.optimize(expr2)),
            Expression::Union(expr1, expr2) => Expression::Union(optimizer.optimize(expr1), optimizer.optimize(expr2)),
            Expression::Rename(expression, old_columns, new_columns) => Expression::Rename(optimizer.optimize(expression), old_columns, new_columns),
            _ => panic!("Unsupported expression, please run UnfoldComplexExpression")
        }
    )

}

/**
 * Automatically detect columns for following passes.
 */
pub struct DetectLoadColumnsOptimizer { }
impl Optimizer for DetectLoadColumnsOptimizer {
    fn optimize(&self, expression: Box<Expression>) -> Box<Expression> {
        if let Expression::Load(filename, None) = *expression { // load expression with no columns detected
            let mut rdr = Reader::from_path(&filename).unwrap();
            let mut columns = HashSet::new();
        
            for header in rdr.headers().unwrap().into_iter() {
                columns.insert(String::from(header));
            }

            Box::new(Expression::Load(filename, Some(columns)))
        } else {
            visit_children(self, expression)
        }
    }
}
/**
 * Compute the columns exposed by a given expression.
 */
fn get_exposed_columns(expression: &Box<Expression>) -> HashSet<String> {
    match &**expression {
        // Si on n'a pas besoin de tous les fields après, on regarde si on a besoin de nouveau fields pour la condition
        Expression::Select(expression_from, _) => get_exposed_columns(&expression_from),
        Expression::Project(_, columns) => columns.iter().cloned().collect(),
        Expression::Product(expr1, expr2) => {
            // Pour les product, on dit qu'on "utilise" un sur ensemble de fields, et on corrige les problèmes dans les load et rename
            let mut fields1 = get_exposed_columns(&expr1);
            let fields2 = get_exposed_columns(&expr2);

            fields1.extend(fields2);

            fields1
        },
        Expression::Except(expr1, _) => get_exposed_columns(&expr1),
        Expression::Union(expr1, _) => get_exposed_columns(&expr1),
        Expression::Rename(expression, old_columns, new_columns) => {
            let mut fields = get_exposed_columns(&expression);

            for i in 0..old_columns.len() {
                fields.remove(&old_columns[i]);
                fields.insert(new_columns[i].clone());
            }

            fields
        },
        Expression::Load(_, columns) => columns.as_ref().unwrap().iter().cloned().collect(),
        _ => panic!("Unsupported expression, please run UnfoldComplexExpression")
    }
}

fn columns_used_in_condition(condition: &Box<Condition>, fields: &mut HashSet<String>) {
    match condition.as_ref() {
        Condition::Not(c) => columns_used_in_condition(c, fields),
        Condition::And(c1, c2) | Condition::Or(c1, c2) => {
            columns_used_in_condition(c1, fields);
            columns_used_in_condition(c2, fields);
        },
        Condition::Equal(v1, v2) | Condition::Less(v1, v2) | Condition::More(v1, v2) => {
            match v1 {
                Value::Column(s) => { fields.insert(s.clone()); () },
                _ => ()
            }
            match v2 {
                Value::Column(s) => { fields.insert(s.clone()); () },
                _ => ()
            }
        }
    }
}

fn rename_value(value: Value, rename_map: &HashMap<String, String>) -> Value {
    match value {
        Value::Column(ref s) => { 
            match rename_map.get(s) {
                None => value,
                Some(new_name) => Value::Column(new_name.clone())

            }
        },
        _ => value
    }
}

fn rename_in_condition(condition: Box<Condition>, rename_map: &HashMap<String, String>) -> Box<Condition> {
    match *condition {
        Condition::Not(c) => Box::new(Condition::Not(rename_in_condition(c, rename_map))),
        Condition::And(c1, c2) => Box::new(Condition::And(rename_in_condition(c1, rename_map), rename_in_condition(c2, rename_map))),
        Condition::Or(c1, c2) => Box::new(Condition::And(rename_in_condition(c1, rename_map), rename_in_condition(c2, rename_map))),
        Condition::Equal(v1, v2) => Box::new(Condition::Equal(rename_value(v1, rename_map), rename_value(v2, rename_map))),
        Condition::Less(v1, v2) => Box::new(Condition::Less(rename_value(v1, rename_map), rename_value(v2, rename_map))),
        Condition::More(v1, v2) => Box::new(Condition::More(rename_value(v1, rename_map), rename_value(v2, rename_map))),
    }
}

/**
 * Try to apply projections as early as possible.
 * 
 * Takes as input the expression to transform and the fields that must be kept (None if all of them).
 * Returns the updated expressions.
 */
fn apply_projections_early(expression: Box<Expression>, fields: Option<HashSet<String>>) -> Box<Expression> {
    match *expression {
        // Si on n'a pas besoin de tous les fields après, on regarde si on a besoin de nouveau fields pour la condition
        Expression::Select(expression_from, condition) if fields.is_some() => {
            let mut fields_set = fields.unwrap();

            let mut used_in_condition = HashSet::new();
            columns_used_in_condition(&condition, &mut used_in_condition);

            // Si on utilise une colonne qui n'est pas requise par la suite, on rajoute une projection
            let mut projection_required = false;
            for condition_field in &used_in_condition {
                if !fields_set.contains(condition_field) {
                    projection_required = true;
                }
            }

            let project_on = fields_set.iter().cloned().collect::<Vec<String>>();
            fields_set.extend(used_in_condition);

            let expression = Box::new(Expression::Select(apply_projections_early(expression_from, Some(fields_set)), condition));

            if projection_required {
                Box::new(Expression::Project(expression, project_on))
            } else {
                expression
            }
        },
        Expression::Select(expression_from, condition) => Box::new(Expression::Select(apply_projections_early(expression_from, fields), condition)),
        Expression::Project(expression_from, columns) => 
        {
            // Quand on a un project, les éléments utilisés correspondent exactement aux éléments du project
            let mut fields = HashSet::new();
            for column in columns {
                fields.insert(column);
            }

            // On remonte les project, donc rien à faire ici
            apply_projections_early(expression_from, Some(fields))
        },
        Expression::Product(expr1, expr2) => {
            // Pour les product, on dit qu'on "utilise" un sur ensemble de fields, et on corrige les problèmes dans les load et rename
            let fields2 = fields.clone();
            let final_expr1 = apply_projections_early(expr1, fields);
            let final_expr2 = apply_projections_early(expr2, fields2);

            Box::new(Expression::Product(final_expr1, final_expr2))
        },
        Expression::Except(expr1, expr2) => {
            let fields2 = fields.clone();

            Box::new(Expression::Except(apply_projections_early(expr1, fields), apply_projections_early(expr2, fields2)))
        },
        Expression::Union(expr1, expr2) => {
            let fields2 = fields.clone();

            Box::new(Expression::Union(apply_projections_early(expr1, fields), apply_projections_early(expr2, fields2)))
        },
        Expression::Rename(expression, old_columns, new_columns) if fields.is_some() => {
            let mut fields_set = fields.unwrap();

            for i in 0..old_columns.len() {
                if fields_set.contains(&new_columns[i]) {
                    fields_set.remove(&new_columns[i]);
                    fields_set.insert(old_columns[i].clone());
                } else {
                    fields_set.remove(&old_columns[i]);
                }
            }

            Box::new(Expression::Rename(apply_projections_early(expression, Some(fields_set)), old_columns, new_columns))
        },
        Expression::Rename(expression, old_columns, new_columns) => Box::new(Expression::Rename(apply_projections_early(expression, fields), old_columns, new_columns)),
        Expression::Load(_, ref columns) if fields.is_some() => {
            let fields_set = fields.unwrap();
            // DetectLoadColumnsOptimizer must be executed before
            let project_on = columns.as_ref().unwrap().into_iter().filter(|column| fields_set.contains(*column)).cloned().collect::<Vec<_>>();

            // On ajoute une projection que si cela limite réellement les champs dispo
            if project_on.len() != columns.as_ref().unwrap().len() {
                Box::new(Expression::Project(expression, project_on))
            } else {
                expression
            }
        },
        Expression::Load(_, _) => expression,
        _ => panic!("Unsupported expression, please run UnfoldComplexExpression")
    }
}

/**
 * Try to apply projections earlier
 */
pub struct ApplyProjectionsEarlyOptimizer { }
impl Optimizer for ApplyProjectionsEarlyOptimizer {
    fn optimize(&self, expression: Box<Expression>) -> Box<Expression> {
        apply_projections_early(expression, None)
    }
}

/**
 * Try to push down selections and merge selections.
 */
fn push_down_selections(mut expression: Box<Expression>, mut selections: Vec<(Box<Condition>, HashSet<String>)>) -> Box<Expression> {
    match *expression {
        // Si on n'a pas besoin de tous les fields après, on regarde si on a besoin de nouveau fields pour la condition
        Expression::Select(expression_from, condition) => {
            let mut fields = HashSet::new();
            columns_used_in_condition(&condition, &mut fields);
            selections.push((condition, fields));

            push_down_selections(expression_from, selections)
        },
        Expression::Project(expression_from, columns) => Box::new(Expression::Project(push_down_selections(expression_from, selections), columns)),
        Expression::Product(expr1, expr2) => {
            let fields1 = get_exposed_columns(&expr1);

            // on voit si on peut remonter certaines conditions
            let (selections1, selections): (Vec<_>, Vec<_>) = selections.into_iter().partition(|(_, fields)| fields.iter().all(|field| fields1.contains(field)));
            let (selections2, selections): (Vec<_>, Vec<_>) = selections.into_iter().partition(|(_, fields)| fields.iter().all(|field| !fields1.contains(field)));

            let mut new_expr = Box::new(Expression::Product(push_down_selections(expr1, selections1), push_down_selections(expr2, selections2)));

            for (condition, _) in selections {
                new_expr = Box::new(Expression::Select(new_expr, condition));
            }

            new_expr
        }, 
        Expression::Except(expr1, expr2) => Box::new(Expression::Except(
            push_down_selections(expr1, selections.clone()),
            push_down_selections(expr2, selections)
        )),
        Expression::Union(expr1, expr2) => Box::new(Expression::Union(
            push_down_selections(expr1, selections.clone()),
            push_down_selections(expr2, selections)
        )),
        Expression::Rename(expression, old_columns, new_columns) => {
            let mut rename_map = HashMap::new();
            for i in 0..old_columns.len() {
                rename_map.insert(new_columns[i].clone(), old_columns[i].clone());
            }

            let updated_selections = selections.into_iter().map(|(condition, fields)| {
                (rename_in_condition(condition, &rename_map), fields.into_iter().map(|field| {
                    match rename_map.get(&field) {
                        None => field,
                        Some(new_name) => new_name.clone(),
                    }
                }).collect())
            }).collect();

            Box::new(Expression::Rename(push_down_selections(expression, updated_selections), old_columns, new_columns))
        },
        Expression::Load(_, _) => {
            // Reapply selections
            for (condition, _) in selections {
                expression = Box::new(Expression::Select(expression, condition));
            }

            expression
        },
        _ => panic!("Unsupported expression, please run UnfoldComplexExpression")
    }
}

pub struct PushDownSelectionsOptimizer { }
impl Optimizer for PushDownSelectionsOptimizer {
    fn optimize(&self, expression: Box<Expression>) -> Box<Expression> {
        push_down_selections(expression, Vec::new())
    }
}

/**
 * To simplify we make a few assumptions, on the order of optimizations
 * before this one is executed:
 * 
 * - First, selections are pushed down.
 * - Then, projections are pushed down.
 * 
 * This ensures that when recursively unfolding expressions, we will first see renaming, then projection, then selection, then another projection, then load.
 */
pub struct FoldComplexExpressionsOptimizer { }
impl Optimizer for FoldComplexExpressionsOptimizer {
    fn optimize(&self, mut expression: Box<Expression>) -> Box<Expression> {

        let mut project_on = None;
        let mut rename = None;
        let mut selection = None;

        let mut cont = true;
        while cont {
            match *expression {
                Expression::Select(expr, condition) if selection.is_none() => {
                    selection = Some(condition);
                    expression = expr;
                },
                Expression::Project(expr, attrs) => {
                    if project_on.is_none() {
                        project_on = Some(attrs);
                    }
                    expression = expr;
                },
                Expression::Rename(expr, old_attrs, new_attrs) if project_on.is_none() && selection.is_none() => {
                    rename = Some((old_attrs, new_attrs));
                    expression = expr;
                },
                _ => cont = false
            }
        }

        // on fold que si on a trouvé une sélection
        match *expression {
            Expression::Load(filename, fields) if selection.is_some() => {
                let mut fields = fields.unwrap();
                let condition = selection.unwrap();

                if let Some(project_on_fields) = project_on {
                    fields = project_on_fields.into_iter().collect::<HashSet<_>>();
                }

                let (mut old_attrs, mut new_attrs) = rename.unwrap_or((Vec::new(), Vec::new()));
                for attr in &old_attrs {
                    fields.remove(attr);
                }
                for still_there in fields {
                    old_attrs.push(still_there.clone());
                    new_attrs.push(still_there);
                }

                Box::new(Expression::ReadSelectProjectRename(filename, condition, old_attrs, new_attrs))
            },
            // TODO: add JoinProjectRename support
            // Expression::Product(...) =>
            _ => { // Sinon, on abort et on visite les enfants
                expression = visit_children(self, expression);

                if !selection.is_none() {
                    expression = Box::new(Expression::Select(expression, selection.unwrap()));
                }
                if !project_on.is_none() {
                    expression = Box::new(Expression::Project(expression, project_on.unwrap()));
                }
                if !rename.is_none() {
                    let (old, new) = rename.unwrap();
                    expression = Box::new(Expression::Rename(expression, old, new));
                }

                expression
            }
        }
    }
}