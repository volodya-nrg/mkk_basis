use email_address::EmailAddress;
use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_REGEX1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("failed to compile regex1")
});
thread_local! {
    static EMAIL_REGEX2: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").expect("failed to compile regex2");
}

pub fn is_valid_email(email: &str) -> bool {
    EmailAddress::is_valid(email)
}
