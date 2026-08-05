use crate::{app::App, errors::NBResult};

mod common;
mod packet;
mod app;
mod device;
mod errors;
mod event;
mod thread;
mod request;
mod sender;
mod receiver;
mod protocol;

fn main() -> NBResult<()>{
    let mut app = App::new()?;
    if let Err(err) = app.run() {
        err.handle_error();
        std::process::exit(1);
    }
    Ok(())
}