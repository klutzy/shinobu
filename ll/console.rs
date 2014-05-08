#![allow(non_camel_case_types)]
#![allow(uppercase_variables)]

use std;

use windows::ll::types::*;

// winbase.h

extern "system" {
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE;
}

// wincon.h

// BOOL WINAPI HandlerRoutine(DWORD dwCtrlType);
pub type PHANDLER_ROUTINE = *c_void;

pub struct SECURITY_ATTRIBUTES {
    pub nLength: DWORD,
    pub lpSecurityDescriptor: LPVOID,
    pub bInheritHandle: BOOL,
}

// C: union { WCHAR unicodeChar; CHAR asciiChar; }
pub struct Char {
    pub data: [c_uchar, ..2u],
}

impl Char {
    pub fn unicode_char(&self) -> Option<char> {
        let w = unsafe {
            let pw: *WCHAR = std::cast::transmute(&self.data);
            *pw
        };
        std::char::from_u32(w as u32)
    }

    pub fn UnicodeChar(&mut self) -> *mut WCHAR {
        unsafe { std::cast::transmute(&self) }
    }
    pub fn AsciiChar(&mut self) -> *mut CHAR {
        unsafe { std::cast::transmute(&self) }
    }
}

// C: union {
//     KEY_EVENT_RECORD KeyEvent;
//     MOUSE_EVENT_RECORD MouseEvent;
//     WINDOW_BUFFER_SIZE_RECORD WindowBufferSizeEvent;
//     MENU_EVENT_RECORD MenuEvent;
//     FOCUS_EVENT_RECORD FocusEvent;
// };
pub struct EventRecord {
    pub data: [c_uchar, ..0u],
}

impl EventRecord {
    pub fn KeyEvent(&mut self) -> *mut KEY_EVENT_RECORD {
        unsafe { std::cast::transmute(&self) }
    }
    pub fn MouseEvent(&mut self) -> *mut MOUSE_EVENT_RECORD {
        unsafe { std::cast::transmute(&self) }
    }
    pub fn WindowBufferSizeEvent(&mut self) -> *mut WINDOW_BUFFER_SIZE_RECORD {
        unsafe { std::cast::transmute(&self) }
    }
    pub fn MenuEvent(&mut self) -> *mut MENU_EVENT_RECORD {
        unsafe { std::cast::transmute(&self) }
    }
    pub fn FocusEvent(&mut self) -> *mut FOCUS_EVENT_RECORD {
        unsafe { std::cast::transmute(&self) }
    }
}

pub struct CHAR_INFO {
    pub uChar: Char,
    pub Attributes: WORD,
}

pub struct CONSOLE_CURSOR_INFO {
    pub dwSize: DWORD,
    pub bVisible: BOOL,
}

pub struct CONSOLE_FONT_INFO {
    pub nFont: DWORD,
    pub dwFontSize: COORD,
}

// minimum: vista
//struct CONSOLE_FONT_INFOEX {
//    cbSize: ULONG,
//    nFont: DWORD,
//    dwFontSize: COORD,
//    FontFamily: UINT,
//    FontWeight: UINT,
//    FaceName: [WCHAR, ..LF_FACESIZE];
//}

// minimum: vista
//struct CONSOLE_HISTORY_INFO {
//    cbSize: UINT,
//    HistoryBufferSize: UINT,
//    NumberOfHistoryBuffers: UINT ,
//    dwFlags: DWORD,
//}

pub struct CONSOLE_READCONSOLE_CONTROL {
    pub nLength: ULONG,
    pub nInitialChars: ULONG,
    pub dwCtrlWakeupMask: ULONG,
    pub dwControlKeyState: ULONG,
}

pub struct CONSOLE_SCREEN_BUFFER_INFO {
    pub dwSize: COORD,
    pub dwCursorPosition: COORD,
    pub wAttributes: WORD,
    pub srWindow: SMALL_RECT,
    pub dwMaximumWindowSize: COORD,
}

// minimum: vista
//struct CONSOLE_SCREEN_BUFFER_INFOEX {
//    cbSize: ULONG,
//    dwSize: COORD,
//    dwCursorPosition: COORD,
//    wAttributes: WORD,
//    srWindow: SMALL_RECT,
//    dwMaximumWindowSize: COORD,
//    wPopupAttributes: WORD,
//    bFullscreenSupported: BOOL,
//    ColorTable: [COLORREF, ..16],
//}

// minimum: XP
pub struct CONSOLE_SELECTION_INFO {
    pub dwFlags: DWORD,
    pub dwSelectionAnchor: COORD,
    pub srSelection: SMALL_RECT,
}

pub struct COORD {
    pub X: SHORT,
    pub Y: SHORT,
}

// "internal"
pub struct FOCUS_EVENT_RECORD {
    pub bSetFocus: BOOL,
}

pub struct INPUT_RECORD {
    pub EventType: WORD,
    pub Event: EventRecord,
}

pub struct KEY_EVENT_RECORD {
    pub bKeyDown: BOOL,
    pub wRepeatCount: WORD,
    pub wVirtualKeyCode: WORD,
    pub wVirtualScanCode: WORD,
    pub uChar: Char,
    pub dwControlKeyState: DWORD,
}

// "internal"
pub struct MENU_EVENT_RECORD {
    pub dwCommandId: UINT,
}

pub struct MOUSE_EVENT_RECORD {
    pub dwMousePosition: COORD,
    pub dwButtonState: DWORD,
    pub dwControlKeyState: DWORD,
    pub dwEventFlags: DWORD,
}

pub struct SMALL_RECT {
    pub Left: SHORT,
    pub Top: SHORT,
    pub Right: SHORT,
    pub Bottom: SHORT,
}

pub struct WINDOW_BUFFER_SIZE_RECORD {
    pub dwSize: COORD,
}

extern "system" {
    pub fn AllocConsole() -> BOOL;

    pub fn CreateConsoleScreenBuffer(
        arg1: DWORD, arg2: DWORD, arg3: *SECURITY_ATTRIBUTES, arg4: DWORD, arg5: LPVOID
    ) -> HANDLE;

    pub fn FillConsoleOutputAttribute(
        arg1: HANDLE, arg2: WORD, arg3: DWORD, arg4: COORD, arg5: PDWORD
    ) -> BOOL;

    pub fn FillConsoleOutputCharacterW(
        arg1: HANDLE, arg2: WCHAR, arg3: DWORD, arg4: COORD, arg5: PDWORD
    ) -> BOOL;

    pub fn FlushConsoleInputBuffer(arg1: HANDLE) -> BOOL;

    pub fn FreeConsole() -> BOOL;

    pub fn GenerateConsoleCtrlEvent(arg1: DWORD, arg2: DWORD) -> BOOL;

    pub fn GetConsoleCP() -> UINT;

    pub fn GetConsoleCursorInfo(arg1: HANDLE, arg2: *mut CONSOLE_CURSOR_INFO) -> BOOL;

    pub fn GetConsoleMode(arg1: HANDLE, arg2: PDWORD) -> BOOL;

    pub fn GetConsoleOutputCP() -> UINT;

    pub fn GetConsoleScreenBufferInfo(arg1: HANDLE, arg2: *mut CONSOLE_SCREEN_BUFFER_INFO) -> BOOL;

    pub fn GetConsoleTitleW(arg1: LPWSTR, arg2: DWORD) -> DWORD;

    pub fn GetConsoleWindow() -> HWND;

    pub fn GetLargestConsoleWindowSize(arg1: HANDLE) -> COORD;

    pub fn GetNumberOfConsoleInputEvents(arg1: HANDLE, arg2: PDWORD) -> BOOL;

    pub fn GetNumberOfConsoleMouseButtons(arg1: PDWORD) -> BOOL;

    pub fn PeekConsoleInputW(arg1: HANDLE, arg2: *mut INPUT_RECORD, arg3: DWORD, arg4: PDWORD) -> BOOL;

    pub fn ReadConsoleW(arg1: HANDLE, arg2: *mut c_void, arg3: DWORD, arg4: PDWORD, arg5: PVOID) -> BOOL;

    pub fn ReadConsoleInputW(arg1: HANDLE, arg2: *mut INPUT_RECORD, arg3: DWORD, arg4: PDWORD) -> BOOL;

    pub fn ReadConsoleOutputAttribute(
        arg1: HANDLE, arg2: LPWORD, arg3: DWORD, arg4: COORD, arg5: LPDWORD
    ) -> BOOL;

    pub fn ReadConsoleOutputCharacterW(
        arg1: HANDLE, arg2: LPWSTR, arg3: DWORD, arg4: COORD, arg5: PDWORD
    ) -> BOOL;

    pub fn ReadConsoleOutputW(
        arg1: HANDLE, arg2: *mut CHAR_INFO, arg3: COORD, arg4: COORD, arg5: *mut SMALL_RECT
    ) -> BOOL;

    pub fn ScrollConsoleScreenBufferW(
        arg1: HANDLE, arg2: *SMALL_RECT, arg3: *SMALL_RECT, arg4: COORD, arg5: *CHAR_INFO
    ) -> BOOL;

    pub fn SetConsoleActiveScreenBuffer(arg1: HANDLE) -> BOOL;

    pub fn SetConsoleCP(arg1: UINT) -> BOOL;

    pub fn SetConsoleCtrlHandler(arg1: PHANDLER_ROUTINE, arg2: BOOL) -> BOOL;

    pub fn SetConsoleCursorInfo(arg1: HANDLE, arg2: *CONSOLE_CURSOR_INFO) -> BOOL;

    pub fn SetConsoleCursorPosition(arg1: HANDLE, arg2: COORD) -> BOOL;

    pub fn SetConsoleMode(arg1: HANDLE, arg2: DWORD) -> BOOL;

    pub fn SetConsoleOutputCP(arg1: UINT) -> BOOL;

    pub fn SetConsoleScreenBufferSize(arg1: HANDLE, arg2: COORD) -> BOOL;

    pub fn SetConsoleTextAttribute(arg1: HANDLE, arg2: WORD) -> BOOL;

    pub fn SetConsoleTitleW(arg1: LPCWSTR) -> BOOL;

    pub fn SetConsoleWindowInfo(arg5: HANDLE, arg2: BOOL, arg3: *SMALL_RECT) -> BOOL;

    pub fn WriteConsoleW(
        arg5: HANDLE, arg2: *VOID, arg3: DWORD, arg4: PDWORD, arg5: PVOID
    ) -> BOOL;

    pub fn WriteConsoleInputW(
        arg1: HANDLE, arg2: *INPUT_RECORD, arg3: DWORD, arg4: PDWORD
    ) -> BOOL;

    pub fn WriteConsoleOutputW(
        arg1: HANDLE, arg2: *CHAR_INFO, arg3: COORD, arg4: COORD, arg5: *SMALL_RECT
    ) -> BOOL;

    pub fn WriteConsoleOutputAttribute(
        arg1: HANDLE, arg2: *WORD, arg3: DWORD, arg4: COORD, arg5: PDWORD
    ) -> BOOL;

    pub fn WriteConsoleOutputCharacterW(
        arg1: HANDLE, arg2: LPCWSTR, arg3: DWORD, arg4: COORD, arg5: PDWORD
    ) -> BOOL;
}
