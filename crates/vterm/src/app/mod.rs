use ::anyhow::{Context, Result};

pub use self::{app_state::State, application::Application};

mod app_state;
mod application;

pub fn run_application<S: State>() -> Result<()> {
    let result = Application::<S>::new()
        .context("failed to construct the application!")?
        .run()
        .context("application exited with an error");

    if let Err(ref error) = result {
        log::error!(
            "Application exited unsuccessfully!\n{:?}\n\nroot cause: {:?}",
            error,
            error.root_cause()
        );
    }
    result
}
