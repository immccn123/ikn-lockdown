#[macro_use]
extern crate windows_service;

use std::ffi::OsString;
use windows_service::service_dispatcher;

mod service;

define_windows_service!(ffi_service_main, service_main);

fn service_main(arguments: Vec<OsString>) {
    if let Err(_e) = service::run(&arguments) {
        return;
    }
}

fn main() -> Result<(), windows_service::Error> {
    service_dispatcher::start("lockdown-service", ffi_service_main)?;

    Ok(())
}
