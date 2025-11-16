use std::sync::OnceLock;

#[derive(Debug)]
pub(crate) struct Status {
    pub crimson_hash_error: bool,
    pub dmc3_hash_error: bool,
}

pub static STATUS: OnceLock<Status> = OnceLock::new();

#[unsafe(export_name = "get_loader_status")]
pub extern "C" fn get_loader_status() -> &'static Status {
    STATUS.get().unwrap()
}