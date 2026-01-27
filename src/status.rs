use randomizer_utilities::dmc::loader_parser::LoaderStatus;
use std::sync::OnceLock;

pub static STATUS: OnceLock<LoaderStatus> = OnceLock::new();

// Allows DMC1 and 3 to get the loader status so the overlay can display potential issues
#[unsafe(no_mangle)]
pub extern "C" fn get_loader_status() -> *const LoaderStatus {
    STATUS.get().unwrap()
}
