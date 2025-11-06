/// Checks to see if DDMK is loaded
pub fn is_ddmk_loaded() -> bool {
    randomizer_utilities::is_library_loaded("Mary.dll")
}

/// Checks to see if Crimson is loaded
pub fn is_crimson_loaded() -> bool {
    randomizer_utilities::is_library_loaded("Crimson.dll")
}