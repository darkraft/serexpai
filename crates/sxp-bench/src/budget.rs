//! D-06 reduction. Evaluation ids are frozen. Tuning rows go first.

use crate::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BudgetPlan {
    pub tuning_ids: Vec<String>,
    pub evaluation_ids: Vec<String>,
    pub cap: u32,
}

/// On excess, drop tuning rows and keep every evaluation id. The cap stays.
pub fn reduce_budget(over_budget: bool, plan: &BudgetPlan) -> BudgetPlan {
    if !over_budget {
        return plan.clone();
    }
    BudgetPlan {
        tuning_ids: Vec::new(),
        evaluation_ids: plan.evaluation_ids.clone(),
        cap: plan.cap,
    }
}

/// Lower the codebook cap after the corpus shrink. Evaluation ids stay put.
pub fn lower_cap(plan: &BudgetPlan) -> BudgetPlan {
    let cap = if plan.cap > 1 { plan.cap / 2 } else { plan.cap };
    BudgetPlan {
        tuning_ids: plan.tuning_ids.clone(),
        evaluation_ids: plan.evaluation_ids.clone(),
        cap,
    }
}

/// A plan that changes the evaluation id set is a new experiment.
pub fn assert_same_evaluation(before: &[String], after: &[String]) -> Result<(), Error> {
    let mut left = before.to_vec();
    let mut right = after.to_vec();
    left.sort();
    right.sort();
    if left == right {
        Ok(())
    } else {
        Err(Error::NewExperiment)
    }
}
