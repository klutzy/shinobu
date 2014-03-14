#[feature(globs, macro_rules)];
#[no_uv];

extern crate native;
extern crate windows = "rust-windows";

pub mod ll {
    pub mod console;
    pub mod process;
}

mod ui;
mod console;

fn main() {
    windows::window::init_window_map();

    let instance = windows::instance::Instance::main_instance();
    let window = ui::ConsoleWindow::new(instance, ~"Shinobu");
    debug!("main: {:?}", main);
    let window = window.expect("oh my");

    window.show(1);
    window.update();

    let exit_code = windows::main_window_loop();
    std::os::set_exit_status(exit_code as int);
}

#[start]
fn start(argc: int, argv: **u8) -> int {
    native::start(argc, argv, proc() {
        main();
    })
}
