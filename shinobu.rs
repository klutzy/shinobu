#[feature(globs)];

extern mod windows = "rust-windows";

use std::ptr;

use windows::ll::*;
use windows::wchar::ToCU16Str;

pub mod ll {
    pub mod console;
    pub mod process;
}

fn main() {
    let cmd = "ls"; // TODO test
    let mut cmd_line = cmd;
    let proc_attrs = ll::process::SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<ll::process::SECURITY_ATTRIBUTES>() as DWORD,
        lpSecurityDescriptor: ptr::mut_null(),
        bInheritHandle: 1,
    };
    let thread_attrs = ll::process::SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<ll::process::SECURITY_ATTRIBUTES>() as DWORD,
        lpSecurityDescriptor: ptr::mut_null(),
        bInheritHandle: 1,
    };

    let in_handle = unsafe { ll::console::GetStdHandle(-10 as DWORD) };
    let out_handle = unsafe { ll::console::GetStdHandle(-11 as DWORD) };
    let err_handle = unsafe { ll::console::GetStdHandle(-12 as DWORD) };

    let startup_info = ll::process::STARTUPINFO {
        cb: std::mem::size_of::<ll::process::STARTUPINFO>() as DWORD,
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
        dwFlags: 0,
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

    let handle = unsafe {
        cmd_line.with_c_u16_str_mut(|cmd_line| {
            ll::process::CreateProcessW(
                ptr::null(), cmd_line, &proc_attrs, &thread_attrs,
                1, 0, ptr::mut_null(), ptr::null(), &startup_info, &mut proc_info
            )
        })
    };

    if handle == 0 {
        let err = unsafe { ll::process::GetLastError() };
        debug!("err: {:?}", err);
        return;
    }
}
