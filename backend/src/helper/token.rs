use rand::distributions::{Alphanumeric, DistString};

pub fn alphanumeric(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}
