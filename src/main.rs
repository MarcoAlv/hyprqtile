mod cli;
mod ipc;
mod prelude;
use cli::HyprQtileArgs;
use ipc::*;
use prelude::*;

fn main() -> Result<()> {
    let args = HyprQtileArgs::parse_args();
    let verbose = args.verbose;

    match args {
        HyprQtileArgs {
            workspace: Some(workspace),
            ..
        } => move_to(workspace, verbose)?,
        HyprQtileArgs { previous: true, .. } => move_to_previous(verbose)?,
        HyprQtileArgs { next: true, .. } => move_to_next(verbose)?,
        _ => (),
    }

    Ok(())
}
