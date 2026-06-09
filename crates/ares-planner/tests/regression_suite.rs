// This file acts as the entrypoint for cargo test so it discovers the regression sub-modules
mod regression {
    mod regression_cycle_detection;
    mod regression_replanning;
    mod regression_scoring_order;
    mod regression_workflow_mapping;
}
