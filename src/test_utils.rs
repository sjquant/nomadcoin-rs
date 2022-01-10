use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
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

pub fn test_db() -> PickleDb {
    let temp_path = env::temp_dir().join(format!("{}.db", random_string(32)));
    PickleDb::new(
        temp_path,
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Bin,
    )
}
