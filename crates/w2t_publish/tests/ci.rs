use gh_workflow_tailcall::*;

#[test]
fn generate_ci_workflow() {
    let workflow = Workflow::default().auto_release(true).auto_fix(true);
    workflow.generate().unwrap();
}
