use std::ptr;
use std::default::Default;
use std::cell::RefCell;
use std;

use windows::window::{Window, WindowImpl, WindowParams, WndClass};
use windows::window::{OnCreate, OnDestroy, OnPaint};
use windows::window;
use windows::gdi::WindowPaint;
use windows::instance::Instance;
use windows::resource::Image;
use windows::resource;
use windows::ll::{HBRUSH, CREATESTRUCT, LRESULT, LPARAM, UINT, WPARAM, UINT_PTR, HWND, BOOL, RECT};
use windows::font;
use windows;

use super::console;
use super::console::ConsoleProcess;

static TIMER_ID: UINT_PTR = 0x123;

pub struct ConsoleWindow {
    win: Window,
    font: font::Font,
    buf: RefCell<~str>,
    subproc: ConsoleProcess,
}

// extern "stdcall" fn TimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD);
type TIMERPROC = *();

extern "system" {
    fn SetTimer(hWnd: HWND, nIDEvent: UINT_PTR, uElapse: UINT, lpTimerFunc: TIMERPROC) -> UINT_PTR;

    fn InvalidateRect(hWnd: HWND, lpRect: *RECT, bErase: BOOL) -> BOOL;
}

#[path = "wnd_proc_macro.rs"]
mod macro;

wnd_proc!(ConsoleWindow, win, WM_CREATE, WM_DESTROY, WM_PAINT, WM_TIMER)

impl OnCreate for ConsoleWindow {
    fn on_create(&self, _cs: &CREATESTRUCT) -> bool {
        let ret = unsafe {
            SetTimer(self.wnd().wnd, TIMER_ID, 10, ptr::null())
        };
        if ret == 0 {
            return false;
        }

        true
    }
}

impl OnDestroy for ConsoleWindow {}

impl OnPaint for ConsoleWindow {
    fn on_paint(&self) {
        self.with_paint_dc(|dc| {
            dc.select_font(&self.font);

            let mut x = 0;
            let mut y = 0;
            let buf = self.buf.get();
            for c in buf.chars() {
                if c == '\n' {
                    x = 0;
                    y += 1;
                    continue;
                }

                // FIXME temporary magic vals
                let cw = 8;
                let ch = 20;
                dc.text_out((x * cw) as int, (y * ch) as int, c.to_str());

                x += 1; // FIXME wide char
            }
        });
    }
}

// FIXME trait OnTimer
impl ConsoleWindow {
    fn on_timer(&self) {
        let output = self.subproc.read_console();
        let is_changed = self.buf.with(|o| { !output.equiv(o) });
        debug!("on_timer: is_changed {}", is_changed);
        if is_changed {
            self.buf.set(output);
            unsafe {
                InvalidateRect(self.wnd().wnd, ptr::null(), 1);
            }
        }
    }
}

impl ConsoleWindow {
    pub fn new(instance: Instance, title: ~str) -> Option<Window> {
        let mut font_attr: font::FontAttr = Default::default();
        font_attr.pitch = font::FIXED_PITCH;
        //font_attr.char_set = font::HANGUL_CHARSET;
        let font = font::Font::new(&font_attr);
        let font = match font {
            Some(font) => font,
            None => return None,
        };

        let cmd_line = "cmd";

        let subproc = console::create_subprocess(cmd_line);
        let subproc = match subproc {
            Some(subproc) => subproc,
            None => return None,
        };

        let (x, y) = subproc.largest_window_size();
        let _ret = subproc.set_screen_buffer_size(x, y);

        let output = subproc.read_console();

        let wnd_class = WndClass {
            classname: ~"ConsoleWindow",
            style: 0x0001 | 0x0002, // CS_HREDRAW | CS_VREDRAW
            icon: None,
            icon_small: None,
            cursor: Image::load_cursor_resource(32513),
            background: (5 + 1) as HBRUSH,
            menu: resource::MenuResource::null(),
            cls_extra: 0,
            wnd_extra: 0,
        };
        let res = wnd_class.register(instance);
        if !res {
            return None;
        }

        let wproc = ~ConsoleWindow {
            win: Window::null(),
            font: font,
            buf: RefCell::new(output),
            subproc: subproc,
        };

        let win_params = WindowParams {
            window_name: title,
            style: window::WS_OVERLAPPEDWINDOW,
            x: 0,
            y: 0,
            width: 400,
            height: 400,
            parent: Window::null(),
            menu: ptr::mut_null(),
            ex_style: 0,
        };

        let classname = "ConsoleWindow";
        Window::new(instance, Some(wproc as ~WindowImpl), classname, &win_params)
    }
}
