// use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
// use std::env;
// use std::fs;
use std::iter;
// use std::path::Path;

pub fn random_string(len: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .collect::<String>()
}

// pub struct DBResource {
//     path: String,
// }

// impl DBResource {
//     fn new(path: String) -> Self {
//         DBResource { path }
//     }
// }

// impl Drop for DBResource {
//     fn drop(&mut self) {
//         let path = Path::new(&self.path);
//         if path.exists() {
//             fs::remove_file(path).unwrap();
//         }
//     }
// }

// pub fn test_repo() -> (DBResource, PickleDb) {
//     let temp_path = env::temp_dir().join(format!("{}.db", random_string(32)));
//     let path_string = temp_path.clone().into_os_string().into_string().unwrap();

//     let repo =
//     let db = PickleDb::new(
//         temp_path,
//         PickleDbDumpPolicy::AutoDump,
//         SerializationMethod::Bin,
//     );
//     let db_resource = DBResource::new(path_string);

//     // Order is important.
//     // db first dropped, and then db_resource dropped
//     (db_resource, db)
// }
