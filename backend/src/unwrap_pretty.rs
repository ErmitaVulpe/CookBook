/// Not really unwrap since it just stops execution but it's good enough for what i use it for

pub trait UnwrapPretty<T> {
    fn unwrap_pretty(self, message: &str) -> T;
}

impl<T, E> UnwrapPretty<T> for Result<T, E>
where E: std::fmt::Display {
    fn unwrap_pretty(self, message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(err) => {
                eprintln!("{}: {}", message, err);
                std::process::exit(1);
            }
        }
    }
}

impl<T> UnwrapPretty<T> for Option<T> {
    fn unwrap_pretty(self, message: &str) -> T {
        match self {
            Some(value) => value,
            None => {
                eprintln!("{}", message);
                std::process::exit(1);
            }
        }
    }
}
