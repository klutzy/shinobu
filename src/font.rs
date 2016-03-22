use winapi::{self, UINT, LPARAM, LONG, WCHAR, BYTE, DWORD, HFONT};
use user32;
use gdi32;
use std::mem::uninitialized;

pub struct FontInfo {
    pub font: HFONT,
    pub height: usize,
    pub width: usize,
}

impl FontInfo {
    #[allow(dead_code)] // TODO necessary??
    pub fn null() -> FontInfo {
        FontInfo {
            font: 0 as HFONT,
            height: 0,
            width: 0,
        }
    }

    pub fn new(hwnd: winapi::HWND) -> FontInfo {
        #![allow(dead_code, non_snake_case)]

        #[repr(C)]
        struct NEWTEXTMETRICW {
            tmHeight: LONG,
            tmAscent: LONG,
            tmDescent: LONG,
            tmInternalLeading: LONG,
            tmExternalLeading: LONG,
            tmAveCharWidth: LONG,
            tmMaxCharWidth: LONG,
            tmWeight: LONG,
            tmOverhang: LONG,
            tmDigitizedAspectX: LONG,
            tmDigitizedAspectY: LONG,
            tmFirstChar: WCHAR,
            tmLastChar: WCHAR,
            tmDefaultChar: WCHAR,
            tmBreakChar: WCHAR,
            tmItalic: BYTE,
            tmUnderlined: BYTE,
            tmStruckOut: BYTE,
            tmPitchAndFamily: BYTE,
            tmCharSet: BYTE,
            ntmFlags: DWORD,
            ntmSizeEM: UINT,
            ntmCellHeight: UINT,
            ntmAvgWidth: UINT,
        }

        #[repr(C)]
        struct FONTSIGNATURE {
            fsUsb: [DWORD; 4],
            fsCsb: [DWORD; 2],
        }

        #[repr(C)]
        struct NEWTEXTMETRICEXW {
            ntmTm: NEWTEXTMETRICW,
            ntmFontSig: FONTSIGNATURE,
        }

        type TEXTMETRICW = NEWTEXTMETRICW;

        type FONTENUMPROC = extern "system" fn(*const winapi::LOGFONTW, *const TEXTMETRICW,
                winapi::DWORD, winapi::LPARAM) -> winapi::c_int;

        extern "system" {
            fn EnumFontFamiliesExW(hdc: winapi::HDC, logfont: winapi::LPLOGFONTW,
                    proc_: FONTENUMPROC, param: winapi::LPARAM, flags: winapi::DWORD)
                    -> winapi::c_int;
        }

        let dc = unsafe { user32::GetDC(hwnd) };

        // TODO test routine

        let mut logfont_def: winapi::LOGFONTW = unsafe { uninitialized() };

        extern "system" fn enum_font_callback(
            logfont: *const winapi::LOGFONTW,
            _metric: *const TEXTMETRICW,
            _d: winapi::DWORD,
            l: winapi::LPARAM
        ) -> winapi::c_int {
            let logfont: &winapi::LOGFONTW = unsafe { &*logfont };
            if logfont.lfPitchAndFamily & 1 == 1 {
                let len = (0..::std::usize::MAX).position(|i| logfont.lfFaceName[i] == 0).unwrap();
                let name = String::from_utf16_lossy(&logfont.lfFaceName[..len]);
                println!("name: {} pitch {}", name, logfont.lfPitchAndFamily & 1);

                let def = unsafe { &mut *(l as *mut winapi::LOGFONTW) };
                *def = *logfont;
                return 0;
            }
            1
        }

        unsafe {
            let mut logfont: winapi::LOGFONTW = uninitialized();
            logfont.lfFaceName[0] = 0;
            logfont.lfCharSet = 0;
            logfont.lfPitchAndFamily = 0;

            let ptr = &mut logfont_def as *mut winapi::LOGFONTW as LPARAM;
            EnumFontFamiliesExW(dc, &mut logfont, enum_font_callback, ptr, 0);
        }

        let font = unsafe { gdi32::CreateFontIndirectW(&logfont_def) };

        unsafe {
            gdi32::SelectObject(dc, font as winapi::HGDIOBJ); // TODO check null

            let mut tm: winapi::TEXTMETRICW = uninitialized();
            gdi32::GetTextMetricsW(dc, &mut tm); // TODO check null

            let height = tm.tmHeight as usize;
            let width = tm.tmAveCharWidth as usize;
            // let max_width = tm.tmMaxCharWidth as usize;

            println!("pitchandfamily {}", tm.tmPitchAndFamily);
            println!("height {} avgwidth {}", height, width);

            FontInfo {
                font: font,
                height: height,
                width: width,
            }
        }
    }
}

