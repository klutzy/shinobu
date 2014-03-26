use std::libc;
use std::ptr;
use std::mem;
use std::str::from_chars;

use windows::ll::{LPCWSTR, DWORD, HANDLE};
use windows::wchar::ToCU16Str;

use super::ll;
use super::ll::console::{Char, CHAR_INFO, COORD, SMALL_RECT};

extern "system" {
    pub fn CreateFileW(lpFileName: LPCWSTR,
                       dwDesiredAccess: DWORD,
                       dwShareMode: DWORD,
                       lpSecurityAttributes: *ll::process::SECURITY_ATTRIBUTES,
                       dwCreationDisposition: DWORD,
                       dwFlagsAndAttributes: DWORD,
                       hTemplateFile: HANDLE) -> HANDLE;
}

pub fn create_con(s: &str, attrs: &ll::process::SECURITY_ATTRIBUTES) -> HANDLE {
    let s = s.to_c_u16();
    unsafe {
        CreateFileW(s.as_ptr(),
                    libc::GENERIC_READ | libc::GENERIC_WRITE,
                    libc::FILE_SHARE_READ | libc::FILE_SHARE_WRITE,
                    attrs,
                    libc::OPEN_EXISTING,
                    0,
                    ptr::mut_null()
        )
    }
}

pub struct ConsoleProcess {
    in_handle: HANDLE,
    out_handle: HANDLE,
    err_handle: HANDLE,
    proc_id: DWORD,
}

impl ConsoleProcess {
    pub fn new(cmd_line: &str) -> Option<ConsoleProcess> {
        static STARTF_USESTDHANDLES: DWORD = 0x100;

        let def_attrs = ll::process::SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<ll::process::SECURITY_ATTRIBUTES>() as DWORD,
            lpSecurityDescriptor: ptr::mut_null(),
            bInheritHandle: 1,
        };
        let in_handle = create_con("CONIN$", &def_attrs);
        let out_handle = create_con("CONOUT$", &def_attrs);
        let err_handle = create_con("CONOUT$", &def_attrs);

        let startup_info = ll::process::STARTUPINFO {
            cb: mem::size_of::<ll::process::STARTUPINFO>() as DWORD,
            lpReserved: ptr::mut_null(),
            lpDesktop: ptr::mut_null(),
            lpTitle: ptr::mut_null(),
            dwX: 0,
            dwY: 0,
            dwXSize: 0,
            dwYSize: 0,
            dwXCountChars: 0,
            dwYCountChars: 0,
            dwFillAttribute: 0,
            dwFlags: STARTF_USESTDHANDLES,
            wShowWindow: 0,
            cbReserved2: 0,
            lpReserved2: ptr::mut_null(),
            hStdInput: in_handle,
            hStdOutput: out_handle,
            hStdError: err_handle,
        };

        let mut proc_info = ll::process::PROCESS_INFORMATION {
            hProcess: ptr::mut_null(),
            hThread: ptr::mut_null(),
            dwProcessId: 0,
            dwThreadId: 0,
        };

        let mut cmd_line_u = cmd_line.to_c_u16();
        let proc_ret = unsafe {
            ll::process::CreateProcessW(
                ptr::null(), cmd_line_u.as_mut_ptr(), &def_attrs, &def_attrs,
                1, 0, ptr::mut_null(), ptr::null(), &startup_info, &mut proc_info
            )
        };
        if proc_ret == 0 {
            let err = unsafe { ll::process::GetLastError() };
            debug!("err: {:?}", err);
            return None; // FIXME
        }

        let ret = ConsoleProcess {
            in_handle: in_handle,
            out_handle: out_handle,
            err_handle: err_handle,
            proc_id: proc_info.dwProcessId,
        };
        Some(ret)
    }

    #[allow(dead_code)]
    pub fn largest_window_size(&self) -> (i16, i16) {
        let lcoord = unsafe { ll::console::GetLargestConsoleWindowSize(self.out_handle) };
        (lcoord.X, lcoord.Y)
    }

    #[allow(dead_code)]
    pub fn set_screen_buffer_size(&self, x: i16, y: i16) -> bool {
        let lcoord = COORD { X: x, Y: y };
        let ret = unsafe { ll::console::SetConsoleScreenBufferSize(self.out_handle, lcoord) };
        ret != 0
    }

    pub fn read_console(&self) -> ~str {
        let mut buf_info = ll::console::CONSOLE_SCREEN_BUFFER_INFO {
            dwSize: COORD { X: 0, Y: 0 },
            dwCursorPosition: COORD { X: 0, Y: 0 },
            wAttributes: 0,
            srWindow: SMALL_RECT { Left: 0, Top: 0, Right: 0, Bottom: 0 },
            dwMaximumWindowSize: COORD { X: 0, Y: 0 },
        };

        let _ret = unsafe {
            ll::console::GetConsoleScreenBufferInfo(self.out_handle, &mut buf_info)
        };

        let size = buf_info.dwSize;
        let mut buf: ~[CHAR_INFO] = ~[];
        for _ in range(0, (size.X as uint) * (size.Y as uint)) {
            buf.push(CHAR_INFO {
                uChar: Char { data: [0, 0] },
                Attributes: 0,
            });
        }

        let mut small_rect = SMALL_RECT {
            Left: buf_info.srWindow.Left,
            Top: buf_info.srWindow.Top,
            Right: buf_info.srWindow.Right,
            Bottom: buf_info.srWindow.Bottom,
        };

        let ret = unsafe {
            ll::console::ReadConsoleOutputW(
                self.out_handle, buf.as_mut_ptr(), size,
                COORD { X: 0, Y: 0 }, &mut small_rect
            )
        };
        if ret == 0 {
            let err = unsafe { ll::process::GetLastError() };
            fail!("err: {:?}", err);
        }

        let mut output: ~[char] = ~[];
        let mut x = 0;
        //let mut y = 0;
        for b in buf.iter() {
            if b.Attributes & 0x0200 != 0 {
                continue; // trailing byte
            }
            let c = b.uChar.unicode_char();
            let c = c.unwrap();
            if c == '\x00' {
                break;
            }
            output.push(c);
            if x % (size.X as uint) == (size.X as uint - 1) {
                output.push('\r'); // FIXME this is really bogus
                output.push('\n');
                x = 0;
                // y += 1;
            } else {
                x += 1;
                if b.Attributes & 0x0100 != 0 {
                    // leading byte
                    x += 1;
                }
            }
        }
        from_chars(output)
    }
}
