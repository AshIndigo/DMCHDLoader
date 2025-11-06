use std::sync::OnceLock;

#[derive(Debug)]
pub(crate) struct Status {
    pub(crate) crimson_hash_error: bool
}

pub static STATUS: OnceLock<Option<Status>> = OnceLock::new();

pub extern "Rust" fn test() -> &'static Option<Status> {
    STATUS.get().unwrap()
}