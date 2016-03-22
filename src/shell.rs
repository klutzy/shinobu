use std::mem;
use std::ptr;
use unicode_width::UnicodeWidthChar;
use winapi::{self, DWORD};
use kernel32;

use windows::ToCU16Str;

pub struct Char {
    pub c: char, // heck, it should be string.. since printing character may be combined
    pub width: u16, // 0 (invisible), 1 (halfwidth), 2 (fullwidth) TODO: may be longer if multiple chars are merged
    // TODO more attributes
}

pub struct Line {
    pub chars: Vec<Char>,
    width: u16,
}

impl Line {
    pub fn new() -> Line {
        Line {
            chars: Vec::new(),
            width: 0,
        }
    }

    pub fn add(&mut self, c: Char) {
        self.width += c.width;
        self.chars.push(c);
    }
}

/// Manages I/O between the console program and Cygwin shell.
/// Application must register `out_handle` and `err_handle` to its own IOCP.
/// When the handles are ready to read, call `self::on_{out,err}_handle()`.
pub struct CygwinShell {
    // confusion warning: we write data to in_handle and read data from out_handle.
    pub in_handle: winapi::HANDLE,
    pub out_handle: winapi::HANDLE,
    pub err_handle: winapi::HANDLE,

    // console screen
    pub lines: Vec<Line>,
    pub width: u16,
    pub height: u16,

    // data from out_handle
    out_buf: Vec<u8>,
}

fn create_pipe(attrs: &mut winapi::SECURITY_ATTRIBUTES) -> (winapi::HANDLE, winapi::HANDLE) {
    unsafe {
        let mut pipe_r: winapi::HANDLE = ptr::null_mut();
        let mut pipe_w: winapi::HANDLE = ptr::null_mut();
        let _ret = kernel32::CreatePipe(&mut pipe_r, &mut pipe_w, attrs, 0);
        // TODO check ret
        (pipe_r, pipe_w)
    }
}

impl CygwinShell {
    pub fn new() -> CygwinShell {
        // TODO check values; don't remember what they are
        let mut def_attrs = winapi::SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<winapi::SECURITY_ATTRIBUTES>() as DWORD,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 1,
        };
        let in_handles = create_pipe(&mut def_attrs);
        let out_handles = create_pipe(&mut def_attrs);
        let err_handles = create_pipe(&mut def_attrs);

        let mut startup_info = winapi::STARTUPINFOW {
            cb: mem::size_of::<winapi::STARTUPINFOW>() as DWORD,
            lpReserved: ptr::null_mut(),
            lpDesktop: ptr::null_mut(),
            lpTitle: ptr::null_mut(),
            dwX: 0,
            dwY: 0,
            dwXSize: 0,
            dwYSize: 0,
            dwXCountChars: 0,
            dwYCountChars: 0,
            dwFillAttribute: 0,
            dwFlags: winapi::STARTF_USESTDHANDLES,
            wShowWindow: 0,
            cbReserved2: 0,
            lpReserved2: ptr::null_mut(),
            hStdInput: in_handles.0,
            hStdOutput: out_handles.1,
            hStdError: err_handles.1,
        };

        let mut proc_info = winapi::PROCESS_INFORMATION {
            hProcess: ptr::null_mut(),
            hThread: ptr::null_mut(),
            dwProcessId: 0,
            dwThreadId: 0,
        };

        let cmd_line = "cygwin-shim.exe";

        // TODO mutable??
        let mut cmd_line_u = cmd_line.to_c_u16();
        let _proc_ret = unsafe {
            kernel32::CreateProcessW(
                ptr::null(), cmd_line_u.as_mut_ptr(), &mut def_attrs, &mut def_attrs,
                1, 0, ptr::null_mut(), ptr::null(), &mut startup_info, &mut proc_info
            )
        };
        // TODO proc_ret

        CygwinShell {
            in_handle: in_handles.1,
            out_handle: out_handles.0,
            err_handle: err_handles.0,
            // TODO test
            width: 80,
            height: 25,
            lines: Vec::new(),
            out_buf: Vec::new(),
        }
    }

    // this must be called only if out_handle is ready
    #[allow(dead_code)]
    pub fn on_out_handle(&mut self) {
        unsafe {
            let mut buf = [0u8; 1024];
            let mut read_len = 0;
            let ret = kernel32::ReadFile(self.out_handle, buf[..].as_mut_ptr() as *mut winapi::c_void, buf.len() as DWORD, &mut read_len, ptr::null_mut());
            if ret == 0 {
                // TODO
                println!("ReadFile err"); return;
            }

            self.out_buf.extend(&buf[..(read_len as usize)]);
        }

        // TODO read all buf at one time

        self.process_out_buf();
    }

    fn process_out_buf(&mut self) {
        // TODO we must do utf8 check as well as escape sequence check
        // i'm lazy right now, ok?
        let lossy_str = String::from_utf8_lossy(&self.out_buf).to_string();
        self.out_buf.truncate(0);

        if self.lines.len() == 0 {
            self.lines.push(Line::new());
        }

        let mut last_line = self.lines.len() - 1;
        for c in lossy_str.chars() {
            let mut newline = false;

            let c_width = UnicodeWidthChar::width(c).unwrap_or(0) as u16; // TODO 1?
            // basic control characters
            if c == '\r' {
                // TODO ignore for now; we're doing lots of hecks anyway
            } else if c == '\n' {
                newline = true;
            }

            if self.lines[last_line].width > self.width + c_width {
                newline = true;
            }

            if newline {
                self.lines.push(Line::new());
                last_line += 1;
            }

            let c = Char {
                c: c,
                width: c_width,
            };
            self.lines[last_line].add(c);
        }
    }
}
