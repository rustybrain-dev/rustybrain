use std::{cell::RefCell, rc::Rc};

use color_eyre;

use rustybrain_core::config::ConfigLoader;
use rustybrain_gtk::run;

fn main() -> Result<(), String> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install().unwrap();
    let config = ConfigLoader::new().load()?;
    run(Rc::new(RefCell::new(config)));
    Ok(())
}
