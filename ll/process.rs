use windows::ll::*;

pub struct SECURITY_ATTRIBUTES {
    nLength: DWORD,
    lpSecurityDescriptor: LPVOID,
    bInheritHandle: BOOL,
}

pub struct STARTUPINFO {
    cb: DWORD,
    lpReserved: LPWSTR,
    lpDesktop: LPWSTR,
    lpTitle: LPWSTR,
    dwX: DWORD,
    dwY: DWORD,
    dwXSize: DWORD,
    dwYSize: DWORD,
    dwXCountChars: DWORD,
    dwYCountChars: DWORD,
    dwFillAttribute: DWORD,
    dwFlags: DWORD,
    wShowWindow: WORD,
    cbReserved2: WORD,
    lpReserved2: LPBYTE,
    hStdInput: HANDLE,
    hStdOutput: HANDLE,
    hStdError: HANDLE,
}

pub struct PROCESS_INFORMATION {
    hProcess: HANDLE,
    hThread: HANDLE,
    dwProcessId: DWORD,
    dwThreadId: DWORD,
}

extern "system" {
    pub fn GetLastError() -> DWORD;

    pub fn CreateProcessW(
        lpApplicationName: LPCWSTR, lpCommandLine: LPWSTR,
        lpProcessAttributes: *SECURITY_ATTRIBUTES,
        lpThreadAttributes: *SECURITY_ATTRIBUTES,
        bInheritHandles: BOOL, dwCreationFlags: DWORD, lpEnvironment: LPVOID,
        lpCurrentDirectory: LPCWSTR, lpStartupInfo: *STARTUPINFO,
        lpProcessInformation: *mut PROCESS_INFORMATION
    ) -> BOOL;
}
