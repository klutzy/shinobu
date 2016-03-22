// thin wrappers / utilities for Windows
// some portions based on rust-windows

use std::mem;
use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi;
use winapi::{WPARAM, LPARAM, LRESULT, LONG, BOOL, c_int};
use user32;

pub trait OnCreate {
    #[inline(always)]
    fn wm_create(&self, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let cs = unsafe {
            let pcs: *const winapi::CREATESTRUCTW = mem::transmute(lparam);
            &(*pcs)
        };
        let ret = self.on_create(cs);
        if ret {
            return 0 as LRESULT;
        } else {
            return -1 as LRESULT;
        }
    }

    fn on_create(&self, _cs: &winapi::CREATESTRUCTW) -> bool {
        true
    }
}

pub trait OnDestroy {
    #[inline(always)]
    fn wm_destroy(&self, _wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
        self.on_destroy();
        0 as LRESULT
    }

    fn on_destroy(&self) {
        unsafe {
            user32::PostQuitMessage(0 as c_int);
        }
    }
}

pub trait OnPaint {
    #[inline(always)]
    fn wm_paint(&self, _wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
        self.on_paint();
        0 as LRESULT
    }

    fn on_paint(&self) {
    }
}

pub trait OnChar {
    #[inline(always)]
    fn wm_char(&self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        self.on_char(wparam as u16, lparam as u32);
        0 as LRESULT
    }

    fn on_char(&self, _code: u16, _flags: u32) {
    }
}


/// A generic trait for converting a value to a `CU16String`, like `ToCStr`.
pub trait ToCU16Str {
    fn to_c_u16(&self) -> Vec<u16>;
}

impl<'a> ToCU16Str for &'a str {
    fn to_c_u16(&self) -> Vec<u16> {
        let mut t: Vec<u16> = OsStr::new(self).encode_wide().collect();
        t.push(0u16);
        t
    }
}

impl ToCU16Str for String {
    fn to_c_u16(&self) -> Vec<u16> {
        let x : &str = &self;
        x.to_c_u16()
    }
}


pub struct PaintDc {
    pub dc: winapi::HDC,
    pub ps: winapi::PAINTSTRUCT,
    hwnd: winapi::HWND,
}

impl PaintDc {
    pub fn new(hwnd: winapi::HWND) -> Option<PaintDc> {
        let mut ps = winapi::PAINTSTRUCT {
            hdc: ptr::null_mut(),
            fErase: 0 as BOOL,
            rcPaint: winapi::RECT {
                left: 0 as LONG, top: 0 as LONG,
                right: 0 as LONG, bottom: 0 as LONG
            },
            fRestore: 0 as BOOL,
            fIncUpdate: 0 as BOOL,
            rgbReserved: [0 as winapi::BYTE; 32],
        };

        let dc = unsafe { user32::BeginPaint(hwnd, &mut ps) };
        if dc.is_null() {
            return None;
        }

        let pdc = PaintDc {
            dc: dc,
            ps: ps,
            hwnd: hwnd,
        };
        Some(pdc)
    }
}

impl Drop for PaintDc {
    fn drop(&mut self) {
        unsafe { user32::EndPaint(self.hwnd, &self.ps) };
    }
}

