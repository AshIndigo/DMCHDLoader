use std::sync::OnceLock;

#[derive(Debug)]
pub(crate) struct Status {
    // DMC3
    pub dmc3_hash_error: bool,
    pub crimson_hash_error: bool,
    pub ddmk_dmc3_hash_error: bool,
    // DMC1
    pub dmc1_hash_error: bool,
    pub ddmk_dmc1_hash_error: bool,
}

pub static STATUS: OnceLock<Status> = OnceLock::new();

// Allows DMC1 and 3 to get the loader status so the overlay can display potential issues
#[unsafe(no_mangle)]
pub extern "C" fn get_loader_status() -> *const Status {
    STATUS.get().unwrap()
}
