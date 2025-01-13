use rand::distributions::{Alphanumeric, DistString};

pub const AUTH_TOKEN_LEN: &'static usize = &100;

pub fn alphanumeric(len: &usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), *len)
}
