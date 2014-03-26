#[feature(globs, macro_rules, phase)];

#[phase(syntax, link)]
extern crate log;

extern crate getopts;
extern crate windows = "rust-windows";

use std::io::net::tcp::{TcpListener, TcpStream};
use std::io::net::ip::SocketAddr;
use std::io::{Acceptor, Listener};
use std::local_data;
use std::comm;
use std::ptr;
use std::str;
use std::os;
use std::from_str::FromStr;

use getopts::{getopts, optopt};

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
            match key {
                0x00 => continue, // NULL
                0x07 => continue, // BELL
                0x08 => continue, // backspace
                0x09 => continue, // tab
                0x0a => continue, // skip \n. \r is handled below
                0x0b => continue, // vertical tab
                0x0c => continue, // form feed
                0..31 if key != 0x0d => continue, // control seq. ignore for now
                0xff => { // telnet command
                    let cmd = stream.read_byte().unwrap();
                    match cmd {
                        0xf0..0xfa => continue,
                        0xfb..0xfe => { // option negotiation
                            let _option = stream.read_byte().unwrap();
                            continue;
                        }
                        0xff => { // IAC
                            // key == 255
                        }
                        _ => {}
                    }
                }
                _ => {
                    // utf-8 character
                    let char_width = str::utf8_char_width(key);
                    if char_width == 0 {
                        continue;
                    }
                    let mut bytes = [0, ..3];
                    bytes[0] = key;
                    stream.fill(bytes.mut_slice(1, char_width)).unwrap();
                    let s = str::from_utf8_lossy(bytes);
                    let s = s.as_slice().char_at(0);
                    unsafe {
                        // send WM_CHAR to console window
                        // WM_KEYDOWN and WM_KEYUP seems not necessary..
                        static WM_CHAR: UINT = 0x0102;
                        windows::ll::PostMessageW(console_wnd, WM_CHAR, s as WPARAM, 0);
                    }
                }
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

fn watch_console() {
    static EVENT_CONSOLE_CARET: UINT = 0x4001;
    static EVENT_CONSOLE_END_APPLICATION: UINT = 0x4007;
    static WINEVENT_OUTOFCONTEXT: UINT = 0;
    //static WINEVENT_SKIPOWNPROCESS: UINT = 0x02;

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
    let args = os::args();
    let opts = [
        optopt("i", "ip", "", "IP"),
        optopt("p", "port", "", "PORT"),
    ];
    let matches = match getopts(args.tail(), opts) {
        Err(e) => fail!("Bad option: {}", e),
        Ok(m) => m,
    };
    let ip = matches.opt_str("i").unwrap_or(~"127.0.0.1");
    let ip = FromStr::from_str(ip).expect("bad ip address");

    let port = matches.opt_str("p").unwrap_or(~"23");
    let port = FromStr::from_str(port).expect("bad port number");

    let addr = SocketAddr { ip: ip, port: port };

    let cmd_line = if !matches.free.is_empty() {
        (*matches.free.get(0)).clone()
    } else {
        ~"cmd"
    };

    let (sender, receiver) = comm::channel();

    let console_wnd = unsafe { ll::console::GetConsoleWindow() };
    let subproc = console::ConsoleProcess::new(cmd_line).expect("failed to create subprocess");

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

    watch_console();
}
