use rust_shyper_os::*;

static mut TEST: usize = 0;

#[inline(never)]
pub extern "C" fn fun1(mut arg: usize) {
    println!("fun1: arg {}", arg);
    arg += 1;
    fun2(arg)
}

#[inline(never)]
fn fun2(mut arg: usize) {
    println!("fun2: arg {}", arg);
    arg += 1;
    fun3(arg)
}

#[inline(never)]
fn fun3(mut arg: usize) {
    println!("fun3: arg {}", arg);
    arg += 1;
    unsafe { fun4(arg) }
}

#[inline(never)]
unsafe fn fun4(arg: usize) {
    println!("fun4: arg {}", arg);
    if TEST == 0 {
        TEST = 1;
        panic!("panic on func4!!!");
    }
    println!("return to func4, unwind success");
    loop {}
}

#[allow(dead_code)]
pub extern "C" fn test_recover_thread(arg: usize) {
    println!("[TEST] recover arg {}===", arg);
    fun1(arg);
    loop {}
}
