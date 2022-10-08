use alloc::collections::VecDeque;

use spin::{Mutex, Once};

use crate::drivers::uart::getc;
use crate::libs::thread::{thread_yield, current_thread_id, thread_spawn_privilege};

static BUFFER: Once<Mutex<VecDeque<u8>>> = Once::new();

fn buffer() -> &'static Mutex<VecDeque<u8>> {
    match BUFFER.get() {
        None => BUFFER.call_once(|| Mutex::new(VecDeque::new())),
        Some(x) => x,
    }
}

pub fn get_buffer_char() -> u8 {
    let mut buf = buffer().lock();
    match buf.pop_front() {
        None => 0u8,
        Some(c) => c,
    }
}

extern "C" fn input_thread(_arg: usize) {
    debug!("input_thread started Thread [{}]", current_thread_id());
    loop {
        if let Some(c) = getc() {
            let mut buf = buffer().lock();
            buf.push_back(c);
        }
        thread_yield();
    }
}

extern "C" fn shell_thread(_arg: usize) {
    debug!("shell_thread started Thread [{}]", current_thread_id());
    println!(concat!(
        "Welcome to unishyper ...\n\n",
        "You can input \"help\" for more info.\n"
    ));
    loop {
        print!("SHELL> ");
        let cmd = crate::libs::print::getline();
        println!();
        if cmd.trim().is_empty() {
            continue;
        }
        // debug!("get cmd {}", cmd);
        exec_cmd(cmd.as_str());
    }
}

pub fn init() {
    debug!("init shell thread...");
    thread_spawn_privilege(input_thread, 0, "terminal_input");
    thread_spawn_privilege(shell_thread, 0, "terminal");
}

fn exec_cmd(cmd: &str) {
    let mut cmds = cmd.split_whitespace();
    let command = match cmds.next() {
        Some(command) => command,
        None => {
            println!(
                "[warning] command illegal: \"{}\", please input \"help\" for more info.",
                cmd
            );
            return;
        }
    };
    match command {
        "cat" => handle_cat(cmds.next()),
        "free" => crate::mm::dump_mm_usage(),
        "kill" => handle_kill(cmds.next()),
        "ls" => handle_ls(cmds.next()),
        "mkdir" => handle_mkdir(cmds.next()),
        "ps" => crate::libs::thread::list_threads(),
        "run" => handle_run(cmds.next()),
        "help" => print_help(),
        _ => println!(
            "command not found: \"{}\", please input 'help' for more info.",
            cmd
        ),
    }
}

fn handle_cat(_arg: Option<&str>) {
    #[cfg(feature = "fs")]
    match _arg {
        Some(s) => {
            use crate::libs::fs::FAT_ROOT;
            use crate::libs::fs::interface::O_RDONLY;
            let fd = crate::libs::fs::open(format!("{}{}", FAT_ROOT, s).as_str(), O_RDONLY, 0);
            if fd < 0 {
                println!("cat: {}: No such file or directory", s);
                return;
            }
            let mut buf = [0u8; 128];
            loop {
                let read = crate::libs::fs::read(fd, buf.as_mut_ptr(), buf.len());
                let str = core::str::from_utf8(&buf[0..read as usize]).unwrap();
                print!("{}", str);
                if read < 128 {
                    break;
                }
            }
            println!("");
        }
        None => {
            println!("cat: missing operand\nTry 'help' for more information.");
        }
    };
    #[cfg(not(feature = "fs"))]
    println!("[warning] file system is not supported, please enable \"fs\" feature.");
}

fn handle_mkdir(_arg: Option<&str>) {
    #[cfg(feature = "fs")]
    match _arg {
        Some(s) => {
            use crate::libs::fs::FAT_ROOT;
            if crate::libs::fs::create_dir(format!("{}{}", FAT_ROOT, s).as_str()).is_err() {
                println!("mkdir: cannot create directory '{}'.", s);
            }
        }
        None => {
            println!("mkdir: missing operand\nTry 'help' for more information.");
        }
    };
    #[cfg(not(feature = "fs"))]
    println!("[warning] file system is not supported, please enable \"fs\" feature.");
}

fn handle_ls(_arg: Option<&str>) {
    #[cfg(feature = "fs")]
    match _arg {
        Some(s) => {
            use crate::libs::fs::FAT_ROOT;
            if crate::libs::fs::print_dir(format!("{}{}", FAT_ROOT, s).as_str()).is_err() {
                println!("ls: cannot access '{}': No such file or directory", s);
            }
        }
        None => {
            use crate::libs::fs::FAT_ROOT;
            if crate::libs::fs::print_dir(FAT_ROOT).is_err() {
                println!("ls: cannot access root dir, something is wrong with fs");
            }
        }
    };
    #[cfg(not(feature = "fs"))]
    println!("[warning] file system is not supported, please enable \"fs\" feature.");
}

fn handle_kill(arg: Option<&str>) {
    let arg = match arg {
        Some(arg) => arg.parse::<usize>(),
        None => {
            println!(
                "[warning] missing argument in kill [TID], please input \"ps\" for threads info."
            );
            return;
        }
    };
    match arg {
        Ok(tid) => {
            crate::libs::thread::thread_destroy_by_tid(tid);
        }
        Err(_) => {
            println!("[warning] illegal argument in kill, please input \"help\" for more info.");
        }
    }
}

fn handle_run(arg: Option<&str>) {
    let arg = match arg {
        Some(arg) => arg.parse::<usize>(),
        None => {
            println!(
                "[warning] missing argument in run [TID], please input \"ps\" for threads info."
            );
            return;
        }
    };
    match arg {
        Ok(tid) => {
            crate::libs::thread::thread_wake_to_front_by_tid(tid);
        }
        Err(_) => {
            println!("[warning] illegal argument in run, please input \"help\" for more info.");
        }
    }
}

fn print_help() {
    println!(concat!(
        "This is unishyper,\n",
        "a research unikernel targeting a scalable and predictable runtime for embedded devices.\n",
        "List of classes of commands:\n\n",
        "cat [FILE]\t\t-- Concatenate files and print on the standard output, \"fs\" feature is required.\n",
        "free \t\t\t-- Dump memory usage info.\n",
        "kill [TID]\t\t-- Kill target thread according to TID, you can use \"ps\" command to check running threads.\n",
        "ls [DIRECTORY]\t\t-- List information about the FILEs (the current directory by default), \"fs\" feature is required.\n",
        "mkdir [DIRECTORY]\t-- Create the DIRECTORY, if they do not already exist, \"fs\" feature is required.\n",
        "ps \t\t\t-- Report a snapshot of the current threads, you can use \"run [TID]\" to wake the ready ones.\n",
        "run [TID]\t\t-- Run target thread according to TID, you can use \"ps\" command to check available threads.\n",
        "help \t\t\t-- Print this message.\n"
    ));
}
