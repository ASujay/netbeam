use std::process::ExitCode;

use crate::app::try_main;

mod app;
mod common;
mod device;
mod errors;
mod event;
mod packet;
mod protocol;
mod receiver;
mod registry;
mod sender;
mod thread;

fn main() -> ExitCode {
    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            e.handle_error();
            ExitCode::FAILURE
        }
    }
}
