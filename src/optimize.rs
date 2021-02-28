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

/**
 * Try to apply selections as early as possible.
 * 
 * Takes as input the expression to transform and the fields that must be kept (if known).
 * Returns the updated expressions, and the fields detected to allow to traverse cartesian products.
 */
fn apply_selections_early(expression: Box<Expression>, fields: Option<HashSet<String>>) -> (Box<Expression>, Option<HashSet<String>>) {
    match *expression {
        Expression::Select(expression_from, condition) => (expression_from, None),
        Expression::Project(expression_from, columns) => 
        {
            let mut fields = HashSet::new();
            for column in columns {
                fields.insert(column);
            }

            apply_selections_early(expression_from, Some(fields))
        },
        Expression::Product(expr1, expr2) => (expr1, None),
        Expression::Except(expr1, expr2) => {
            (expr1, None)
        },
        Expression::Union(expr1, expr2) => (expr1, None),
        Expression::Rename(expression, old_columns, new_columns) => (expression, None),
        Expression::ReadSelectProjectRename(_, _, _, _) => (expression, None),
        Expression::Load(_, _) => (expression, None),
        _ => (expression, fields)
    }
}