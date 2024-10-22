#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::libs::print::print_arg(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::libs::print::print_arg(format_args!("{}\n", format_args!($($arg)*)));
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

// macro rules for mpk,

// #[macro_export]
// macro_rules! protected_global_var {
// 	/* immutable */
// 	(static $name:ident: $var_type:ty = $val:expr) => {
//                 #[link_section = ".protected_data"]
//                 static $name: $var_type = $val;
//         };
//         (static $name:ident: $var_type:ty) => {
//                 #[link_section = ".protected_data"]
//                 static $name: $var_type = 0;
//         };
//         (pub static $name:ident: $var_type:ty = $val:expr) => {
//                 #[link_section = ".protected_data"]
//                 pub static $name: $var_type = $val;
//         };
//         (pub static $name:ident: $var_type:ty) => {
//                 #[link_section = ".protected_data"]
//                 pub static $name: $var_type = 0;
// 	};

// 	/* mutable */
//         (static mut $name:ident: $var_type:ty = $val:expr) => {
//                 #[link_section = ".protected_data"]
//                 static mut $name: $var_type = $val;
//         };
//         (static mut $name:ident: $var_type:ty) => {
//                 #[link_section = ".protected_data"]
//                 static mut $name: $var_type = 0;
//         };
//         (pub static mut $name:ident: $var_type:ty = $val:expr) => {
//                 #[link_section = ".protected_data"]
//                 pub static mut $name: $var_type = $val;
//         };
//         (pub static mut $name:ident: $var_type:ty) => {
//                 #[link_section = ".protected_data"]
//                 pub static mut $name: $var_type = 0;
//         };
// }
