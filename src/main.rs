extern crate unicode_width;

#[macro_use] extern crate log;
extern crate winapi;
extern crate kernel32;
extern crate user32;
extern crate gdi32;

mod windows;
mod console_window;
mod shell;
mod font;

fn main() {
    let _exit_code = console_window::main_loop();
}
