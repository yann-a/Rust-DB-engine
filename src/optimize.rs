use crate::types::*;
use csv::Reader;
use std::collections::HashSet;

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
            Expression::ReadSelectProjectRename(_, _, _, _) | Expression::Load(_, _) => *expression
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

fn columns_used_in_condition(condition: &Box<Condition>, fields: &mut HashSet<String>) {
    match condition.as_ref() {
        Condition::True | Condition::False => (),
        Condition::And(c1, c2) | Condition::Or(c1, c2) => {
            columns_used_in_condition(c1, fields);
            columns_used_in_condition(c2, fields);
        },
        Condition::Equal(v1, v2) | Condition::Less(v1, v2) | Condition::More(v1, v2) => {
            match v1.as_ref() {
                Value::Column(s) => { fields.insert(s.clone()); () },
                _ => ()
            }
            match v2.as_ref() {
                Value::Column(s) => { fields.insert(s.clone()); () },
                _ => ()
            }
        }
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
        Expression::ReadSelectProjectRename(_, _, _, _) => (expression), // TODO: implement this
        Expression::Load(_, ref columns) if fields.is_some() => {
            let fields_set = fields.unwrap();
            // DetectLoadColumnsOptimizer must be executed before
            let project_on = columns.as_ref().unwrap().into_iter().filter(|column| fields_set.contains(*column)).cloned().collect::<Vec<_>>();

            Box::new(Expression::Project(expression, project_on))
        },
        Expression::Load(_, _) => expression
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