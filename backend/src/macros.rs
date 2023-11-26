mod macro_mod {
    macro_rules! exit_with_error {
        ($($msg:tt)*) => {{
            eprintln!($($msg)*);
            std::process::exit(1);
        }};
    }
    pub(crate) use exit_with_error;

    macro_rules! readln {
        ($($msg:tt)*) => {{
            let mut input = String::new();
            print!($($msg)*);
            std::io::stdout().flush().unwrap();
            std::io::stdin().read_line(&mut input).expect("Failed to read line");
            input.trim().to_owned()
        }};
    }
    pub(crate) use readln;

    macro_rules! readpw {
        ($($msg:tt)*) => {{
            print!($($msg)*);
            std::io::stdout().flush().unwrap();
            rpassword::read_password().unwrap()
        }};
    }
    pub(crate) use readpw;
}

// Export of macros with wanted scope
#[allow(unused_imports)]
pub use macro_mod::*;
