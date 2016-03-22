use std::cell::RefCell;
use std::rc::Rc;
use std::mem;
use std::ptr;
use winapi;
use winapi::{UINT, WPARAM, LPARAM, LRESULT, LONG, DWORD, c_int};
use winapi::{HINSTANCE, HWND, WNDCLASSEXW, MSG, LPMSG, POINT};
use kernel32;
use user32;
use gdi32;

use windows::{OnCreate, OnDestroy, OnPaint, OnChar, PaintDc, ToCU16Str};
use font::FontInfo;
use shell::CygwinShell;

// XXX dont' remember why Rc is necessary..
thread_local!(static CONSOLE: RefCell<Option<Rc<ConsoleWindow>>> = RefCell::new(None));

const CLASSNAME: &'static str = "ConsoleWindow";

pub struct ConsoleWindow {
    hwnd: HWND,
    shell: RefCell<CygwinShell>,
    font_info: FontInfo,
}

impl ConsoleWindow {
    pub fn register(instance: HINSTANCE) {
        let classname = CLASSNAME.to_c_u16();
        let wcex = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as UINT,
            style: winapi::CS_HREDRAW | winapi::CS_VREDRAW,
            lpfnWndProc: Some(ConsoleWindow::wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: (5 + 1) as winapi::HBRUSH,
            lpszMenuName: ptr::null_mut(), // no menu
            lpszClassName: classname.as_ptr(),
            hIconSm: ptr::null_mut(),
        };
        let res = unsafe { user32::RegisterClassExW(&wcex) };
        if res == 0 {
            panic!("RegisterClassExW failed");
        }
    }

    pub fn create(instance: HINSTANCE) -> HWND {
        let classname = CLASSNAME.to_c_u16();
        let title = "ConsoleWindow".to_c_u16();

        let ex_style = 0;
        let style = winapi::WS_OVERLAPPEDWINDOW;
        let x = -1;
        let y = -1;
        let width = 500;
        let height = 400;

        let parent = ptr::null_mut();
        let menu = ptr::null_mut();

        let hwnd = unsafe {
            user32::CreateWindowExW(
                ex_style, classname.as_ptr(), title.as_ptr(), style,
                x as c_int, y as c_int,
                width as c_int, height as c_int,
                parent, menu, instance,
                ptr::null_mut()
            )
        };
        hwnd
    }

    // will be called by `self::wnd_proc()`
    fn new(hwnd: HWND) -> ConsoleWindow {
        let shell = RefCell::new(CygwinShell::new());
        let font_info = FontInfo::new(hwnd);

        ConsoleWindow {
            hwnd: hwnd,
            shell: shell,
            font_info: font_info,
        }
    }

    unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
        let window = CONSOLE.with(|cell| {
            if cell.borrow().is_none() {
                let window = ConsoleWindow::new(hwnd);

                *cell.borrow_mut() = Some(Rc::new(window));
            }
            cell.borrow().clone().unwrap()
        });

        if msg == 0x0001 { return window.wm_create(w, l); }
        if msg == 0x0002 { return window.wm_destroy(w, l); }
        if msg == 0x000F { return window.wm_paint(w, l); }
        if msg == 0x0102 { return window.wm_char(w, l); }

        return user32::DefWindowProcW(hwnd, msg, w, l);
    }

    fn paint_console(&self, dc: winapi::HDC) {
        let font = self.font_info.font;
        unsafe {
            gdi32::SelectObject(dc, font as winapi::HANDLE);
            // TODO ret
        }

        let mut x = 0;
        let mut y = 0;
        for line in &self.shell.borrow().lines {
            for c in &line.chars {
                let w = x * self.font_info.width;
                let h = y * self.font_info.height;
                self.paint_char(dc, w, h, c.c);

                x += c.width as usize;
            }
            x = 0;
            y += 1;
        }
    }

    fn paint_char(&self, dc: winapi::HDC, x: usize, y: usize, s: char) -> bool {
        let mut array = [0u16; 3];
        array[0] = s as u16;
        array[1] = ((s as u32) >> 16) as u16;

        let len = 3;
        let ret = unsafe {
            gdi32::TextOutW(dc, x as winapi::c_int, y as winapi::c_int, array.as_mut_ptr(), len)
        };
        ret != 0
    }
}

impl OnCreate for ConsoleWindow {
    fn on_create(&self, _cs: &winapi::CREATESTRUCTW) -> bool {
        self.shell.borrow_mut().on_out_handle(); // :p XXX
        self.shell.borrow_mut().on_out_handle(); // :p XXX
        true
    }
}

impl OnDestroy for ConsoleWindow {}

impl OnPaint for ConsoleWindow {
    fn on_paint(&self) {
        let pdc = PaintDc::new(self.hwnd).expect("Paint DC");
        self.paint_console(pdc.dc);
    }
}

impl OnChar for ConsoleWindow {
    fn on_char(&self, _code: u16, _flags: u32) {
        // TODO
        println!("on_char code {} ({:x}) flags {} ({:x})", _code, _code, _flags, _flags);
    }
}


pub fn main_loop() -> usize {
    let main_instance = unsafe {
        kernel32::GetModuleHandleW(ptr::null()) as HINSTANCE
    };

    ConsoleWindow::register(main_instance);
    let hwnd = ConsoleWindow::create(main_instance);

    unsafe {
        user32::ShowWindow(hwnd, 1);
        user32::UpdateWindow(hwnd);
    }

    loop {
        let mut msg = MSG {
            hwnd: ptr::null_mut(),
            message: 0 as UINT,
            wParam: 0 as WPARAM,
            lParam: 0 as LPARAM,
            time: 0 as DWORD,
            pt: POINT { x: 0 as LONG, y: 0 as LONG },
        };

        let ret = unsafe {
            user32::GetMessageW(&mut msg as LPMSG, ptr::null_mut(),
                    0 as UINT, 0 as UINT)
        };

        if ret == 0 {
            let exit_code = msg.wParam;
            return exit_code as usize;
        } else {
            unsafe {
                user32::TranslateMessage(&msg as *const MSG);
                user32::DispatchMessageW(&msg as *const MSG);
            }
        }
    }
}
