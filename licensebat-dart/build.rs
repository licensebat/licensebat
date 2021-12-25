use askalono::Store;
use std::{fs::File, path::Path};

const LICENSE_CACHE: &str = "../datasets/license-cache.bin.zstd";
const SPDX_PATH: &str = "../datasets/modules/spdx-license-list-data/json/details";

fn main() {
    create_license_cache_from_spdx();
}

/// Builds a cache file in order to be used later by askalono
fn create_license_cache_from_spdx() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cache_path = root.join(LICENSE_CACHE);
    if cache_path.exists() {
        println!("the cache file already exists. Avoiding rebuild");
        return;
    }

    let spdx_path = root.join(SPDX_PATH);
    let mut store = Store::new();
    store
        .load_spdx(spdx_path.as_path(), true)
        .expect("Couldn't create a store from SPDX data. Have submodules been initialized?");

    let mut cache = File::create(&cache_path).unwrap();
    store.to_cache(&mut cache).unwrap();

    println!("License cache created at: {:?}", cache_path);
}
