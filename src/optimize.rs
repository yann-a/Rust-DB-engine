use crate::types::*;

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