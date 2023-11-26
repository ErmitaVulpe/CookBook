/// - 7 - 50 characters
/// - 1 upper case letter
/// - 1 lower case letter
/// - 1 special from this list: !@#$%^&*()-_=+[]{}\|<>,./?
pub fn is_valid_password(password: &str) -> bool {
    if password.len() < 7 || 50 < password.len() {
        return false;
    }

    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_no_space = ! password.contains(char::is_whitespace);
    let has_special = password.chars().any(|c| "!@#$%^&*()-_=+[]{}\\|<>,./?".contains(c));

    has_lowercase && has_uppercase && has_special && has_no_space
}

/// - 3 - 25 characters
/// - at least one letter
pub fn is_valid_username(username: &str) -> bool {
    let len = username.len();
    if ! (3..=25).contains(&len) {
        return false;
    };

    let has_letter = username.chars().any(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase());

    has_letter
}

/// - 3 - 255 characters
/// - at least one letter
pub fn is_valid_ingredient_name(name: &str) -> bool {
    let len = name.len();
    if ! (3..=255).contains(&len) {
        return false;
    };

    let has_letter = name.chars().any(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase());

    has_letter
}

pub fn is_valid_socket(socket: &str) -> bool {
    use std::net::ToSocketAddrs;

    match socket.to_socket_addrs() {
        Ok(mut iter) => {
            iter.next().filter(|_| iter.next().is_none()).is_some()
        }
        Err(_) => false,
    }
}