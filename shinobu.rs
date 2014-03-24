#[feature(globs, macro_rules, phase)];

#[phase(syntax, link)]
extern crate log;

extern crate windows = "rust-windows";

use std::io::net::tcp::{TcpListener, TcpStream};
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::io::{Acceptor, Listener};
use std::local_data;
use std::comm;
use std::ptr;

use windows::ll::{WPARAM, UINT, DWORD, HWND, LONG};

use console::ConsoleProcess;

pub mod ll {
    use windows::ll::{UINT, DWORD, HMODULE, HANDLE};

    pub mod console;
    pub mod process;

    pub type HWINEVENTHOOK = HANDLE;
    // extern "system" fn WinEventProc(hWinEventHook: HWINEVENTHOOK, event: DWORD,
    //                                 hwnd: HWND, idObject: LONG, idChild: LONG,
    //                                 dwEventThread: DWORD, dwmsEventTim: DWORD);
    pub type WINEVENTPROC = *();

    extern "system" {
        pub fn SetWinEventHook(eventMin: UINT, eventMax: UINT, hmodWinEventProc: HMODULE,
                               lpfnWinEventProc: WINEVENTPROC, idProcess: DWORD, idThread: DWORD,
                               dwflag: UINT) -> HWINEVENTHOOK;
    }
}

mod console;

fn accept_telnet_serv(console_wnd: HWND, mut stream: TcpStream, r: comm::Receiver<~str>) {
    // recv keystrokes
    let stream_ = stream.clone();
    spawn(proc() {
        let mut stream = stream_;
        loop {
            let key = stream.read_byte().unwrap();
            unsafe {
                match key {
                    0x0a => continue, // skip \n. \r is already consumed
                    0xff => {
                        // 0xff 0xf1 ?
                        // drop 3 bytes
                        let _b1 = stream.read_byte().unwrap();
                        let _b2 = stream.read_byte().unwrap();
                        continue;
                    }
                    0..31 if key != 0x0d => continue, // control seq. ignore for now
                    _ => {}
                }
                // send WM_CHAR to console window
                // WM_KEYDOWN and WM_KEYUP seems not necessary..
                static WM_CHAR: UINT = 0x0102;
                windows::ll::PostMessageW(console_wnd, WM_CHAR, key as WPARAM, 0);
            }
        }
    });

    // send console changes
    loop {
        let buf = r.recv();
        stream.write(bytes!("\x1b[2J\x1b[1;1H")).unwrap(); // FIXME reset terminal
        stream.write(buf.as_bytes()).unwrap();
    }
}

local_data_key!(key_data: (comm::Sender<~str>, ConsoleProcess))

extern "system" fn on_console_event(_hWinEventHook: ll::HWINEVENTHOOK, _event: DWORD,
                                    _hwnd: HWND, _idObject: LONG, _idChild: LONG,
                                    _dwEventThread: DWORD, _dwmsEventTim: DWORD) {
    local_data::get(key_data, |data| {
        let (ref sender, ref subproc) = *data.unwrap();
        let output = subproc.read_console();
        sender.send(output);
    });
}

fn watch_console(subproc: ConsoleProcess) {
    static EVENT_CONSOLE_CARET: UINT = 0x4001;
    static EVENT_CONSOLE_END_APPLICATION: UINT = 0x4007;
    static WINEVENT_OUTOFCONTEXT: UINT = 0;
    //static WINEVENT_SKIPOWNPROCESS: UINT = 0x02;

    let _pid = subproc.proc_id;
    let flags = WINEVENT_OUTOFCONTEXT;
    let _hook_handle = unsafe {
        ll::SetWinEventHook(EVENT_CONSOLE_CARET, EVENT_CONSOLE_END_APPLICATION,
                            ptr::mut_null(), on_console_event as *(),
                            0, 0, flags) // FIXME WATCHING ALL PROCESSES OH NO
    };

    let exit_code = windows::main_window_loop();
    std::os::set_exit_status(exit_code as int);
}

fn main() {
    // FIXME make configurable
    let addr = SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 8000 };
    let cmd_line = "cmd";

    let (sender, receiver) = comm::channel();

    let console_wnd = unsafe { ll::console::GetConsoleWindow() };
    let subproc = console::create_subprocess(cmd_line).expect("failed to create subprocess");

    // will be used in extern callback
    local_data::set(key_data, (sender, subproc));

    let listener = TcpListener::bind(addr);
    spawn(proc() {
        let mut acceptor = listener.listen();
        for stream in acceptor.incoming() {
            // only one connection for now
            accept_telnet_serv(console_wnd, stream.unwrap(), receiver);
        }
    });

    watch_console(subproc);
}
