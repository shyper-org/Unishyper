#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::libs::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::libs::print::print_arg(format_args_nl!($($arg)*));
    })
}

#[macro_export]
macro_rules! align_down {
    ($value:expr, $alignment:expr) => {
        ($value) & !($alignment - 1)
    };
}

#[macro_export]
macro_rules! align_up {
    ($value:expr, $alignment:expr) => {
        align_down!($value + ($alignment - 1), $alignment)
    };
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {
    error!("rust_eh_personality called");
    loop {}
}

#[macro_export]
macro_rules! infoheader {
    // This should work on paper, but it's currently not supported :(
    // Refer to https://github.com/rust-lang/rust/issues/46569
    /*($($arg:tt)+) => ({
        info!("");
        info!("{:=^70}", format_args!($($arg)+));
    });*/
    ($str:expr) => {{
        info!("");
        info!("{:=^70}", $str);
    }};
}

#[macro_export]
macro_rules! infoentry {
	($str:expr, $rhs:expr) => (infoentry!($str, "{}", $rhs));
	($str:expr, $($arg:tt)+) => (info!("{:25}{}", concat!($str, ":"), format_args!($($arg)+)));
}

#[macro_export]
macro_rules! infofooter {
    () => {{
        info!("{:=^70}", '=');
        info!("");
    }};
}
