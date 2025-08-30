use super::{selected_rev, Action, OpTrait};
use crate::{
    app::{App, PromptParams, State},
    error::Error,
    git::{get_current_branch_name, is_branch_merged, does_branch_exist},
    item_data::{ItemData, RefKind},
    menu::arg::Arg,
    term::Term,
    Res,
};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct Checkout;
impl OpTrait for Checkout {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let rev = app.prompt(
                term,
                &PromptParams {
                    prompt: "Checkout",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            checkout(app, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Checkout branch/revision".into()
    }
}

fn checkout(app: &mut App, term: &mut Term, rev: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout", rev]);

    app.close_menu();
    app.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct CheckoutNewBranch;
impl OpTrait for CheckoutNewBranch {
    fn get_action(&self, _target: &ItemData) -> Option<Action> {
        Some(Rc::new(|app: &mut App, term: &mut Term| {
            let branch_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Create and checkout branch:",
                    ..Default::default()
                },
            )?;

            checkout_new_branch_prompt_update(app, term, &branch_name)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Checkout new branch".into()
    }
}

fn checkout_new_branch_prompt_update(app: &mut App, term: &mut Term, branch_name: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.args(["checkout", "-b", branch_name]);

    app.close_menu();
    app.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct Delete;
impl OpTrait for Delete {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let default = match target {
            ItemData::Reference {
                kind: RefKind::Branch(branch),
                ..
            } => Some(branch.clone()),
            _ => None,
        };

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let default = default.clone();

            let branch_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Delete",
                    create_default_value: Box::new(move |_| default.clone()),
                    ..Default::default()
                },
            )?;

            delete(app, term, &branch_name)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Delete branch".into()
    }
}

pub fn delete(app: &mut App, term: &mut Term, branch_name: &str) -> Res<()> {
    if branch_name.is_empty() {
        return Err(Error::BranchNameRequired);
    }

    if get_current_branch_name(&app.state.repo).unwrap() == branch_name {
        return Err(Error::CannotDeleteCurrentBranch);
    }

    let mut cmd = Command::new("git");
    cmd.args(["branch", "-d"]);

    if !is_branch_merged(&app.state.repo, branch_name).unwrap_or(false) {
        app.confirm(term, "Branch is not fully merged. Really delete? (y or n)")?;
        cmd.arg("-f");
    }

    cmd.arg(branch_name);

    app.close_menu();
    app.run_cmd(term, &[], cmd)?;
    Ok(())
}

pub(crate) struct Spinoff;
impl OpTrait for Spinoff {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let default = match target {
            ItemData::Reference {
                kind: RefKind::Branch(branch),
                ..
            } => Some(branch.clone()),
            _ => None,
        };

        Some(Rc::new(move |app: &mut App, term: &mut Term| {
            let default = default.clone();

            let new_branch_name = app.prompt(
                term,
                &PromptParams {
                    prompt: "Name for new branch",
                    create_default_value: Box::new(move |_| default.clone()),
                    ..Default::default()
                },
            )?;

            if new_branch_name.is_empty() {
                return Err(Error::BranchNameRequired);
            }

            if does_branch_exist(&app.state.repo, &new_branch_name).unwrap_or(false) {
                app.display_error(&format!("Cannot spin off {new_branch_name}. It already exists"));
            }

            let current_branch = get_current_branch_name(&app.state.repo);

            if current_branch.is_none() {
                app.display_error("No branch checked out");
            }

            let current_branch = current_branch.unwrap();

            if current_branch == new_branch_name {
                // TODO: Update error enum
                return Err(Error::CannotDeleteCurrentBranch);
            }

            let base_commit = unimplemented!();
            let upstream_branch_commit = unimplemented!();

            // Checkout new branch
            let mut cmd = Command::new("git");
            cmd.args(["checkout", "-b", &new_branch_name]);
            app.run_cmd(term, &[], cmd)?;

            if base_commit == upstream_branch_commit {
                app.display_info(&format!("Branch {current_branch} not changed"));
                return Ok(())
            }

            // TODO: Reset the original branch to the common ancestor

            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "Spinoff branch".into()
    }
}

