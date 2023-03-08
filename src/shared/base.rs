use chrono::prelude::Utc;
use serde::de::DeserializeOwned;
use serde_json::from_reader;
use std::fs::File;
use std::path::Path;

pub fn timestamp() -> i64 {
    return Utc::now().timestamp();
}

pub fn load_json<T>(subs: &mut [&str]) -> T
where
    T: DeserializeOwned,
{
    let target_dir = std::env::current_dir().expect("failed on current dir");

    let mut target_path = Path::new(&target_dir).to_path_buf();
    for sub in subs {
        target_path = target_path.join(sub);
    }

    let target_path = String::from(target_path.to_str().unwrap());
    let target_file =
        File::open(&target_path).expect(&format!("file: {} load failed", &target_path));

    from_reader(target_file).expect(&format!("file: {} invalid JSON", &target_path))
}
