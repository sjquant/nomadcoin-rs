use nut::{DBBuilder, DB};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::env;
use std::iter;

pub fn random_string(len: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect::<String>()
}

pub fn test_db() -> DB {
    let temp_path = env::temp_dir().join(format!("{}.db", random_string(32)));
    return DBBuilder::new(temp_path).autoremove(true).build().unwrap();
}
