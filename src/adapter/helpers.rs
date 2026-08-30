use email_address::EmailAddress;
use rand::{RngExt, distr::Alphanumeric};

pub fn is_valid_email(email: &str) -> bool {
    EmailAddress::is_valid(email)
}

// общая ф-ия, нужна и в тестах и в основном коде
pub fn rand_str_limit(len: usize) -> String {
    // rand::rng() - вызовется в своем потоке. У него локальный итератор.
    // Нельзя его создать в одном потоке "let x = rand::rng()", а потом этот x (генератор) вызывать в другом потоке.
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
