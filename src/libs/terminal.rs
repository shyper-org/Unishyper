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
                "command illegal: \"{}\", please input \"help\" for more info.",
                cmd
            );
            return;
        }
    };
    match command {
        "kill" => handle_kill(cmds.next()),
        "memory" => crate::mm::dump_mm_usage(),
        "run" => handle_run(cmds.next()),
        "thread" => crate::libs::thread::list_threads(),
        "help" => print_help(),
        _ => println!(
            "command not found: \"{}\", please input \"help\" for more info.",
            cmd
        ),
    }
}

fn handle_kill(arg: Option<&str>) {
    let arg = match arg {
        Some(arg) => arg.parse::<usize>(),
        None => {
            println!("missing argument in kill [TID], please input \"thread\" for threads info.");
            return;
        }
    };
    match arg {
        Ok(tid) => {
            crate::libs::thread::thread_destroy_by_tid(tid);
        }
        Err(_) => {
            println!("illegal argument in kill, please input \"help\" for more info.");
        }
    }
}

fn handle_run(arg: Option<&str>) {
    let arg = match arg {
        Some(arg) => arg.parse::<usize>(),
        None => {
            println!("missing argument in run [TID], please input \"thread\" for threads info.");
            return;
        }
    };
    match arg {
        Ok(tid) => {
            crate::libs::thread::thread_wake_by_tid(tid);
        }
        Err(_) => {
            println!("illegal argument in run, please input \"help\" for more info.");
        }
    }
}

fn print_help() {
    println!(concat!(
        "This is unishyper,\n",
        "a research unikernel targeting a scalable and predictable runtime for embedded devices.\n",
        "List of classes of commands:\n\n",
        "kill [TID]\t-- Kill target thread according to TID, you can use \"thread\" command to check running threads.\n",
        "memory \t\t-- Dump memory usage info.\n",
        "run [TID]\t-- Run target thread according to TID, you can use \"thread\" command to check available threads.\n",
        "thread \t\t-- List all threads info, you can use \"run [THREAD_ID]\" to wake the ready ones.\n",
        "help \t\t-- Print this message.\n"
    ));
}
