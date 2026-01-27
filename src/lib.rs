mod config;
mod status;

use crate::status::STATUS;
use anyhow::anyhow;
use randomizer_utilities::dmc::loader_parser::LoaderStatus;
use randomizer_utilities::dmc::versions::{Game, Mod, VersionInformation};
use std::ffi::{CString, c_void};
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::{
    AllocConsole, ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle,
    STD_OUTPUT_HANDLE, SetConsoleMode,
};
use windows::Win32::System::LibraryLoader::LoadLibraryA;
use windows::core::{BOOL, GUID, HRESULT, PCSTR};

static mut REAL_DIRECTINPUT8CREATE: Option<
    unsafe extern "system" fn(HINSTANCE, u32, GUID, *mut *mut c_void, *mut c_void) -> HRESULT,
> = None;

fn load_real_dinput8() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let system_path = std::env::var("WINDIR").unwrap_or("C:\\Windows".to_string());
        let real_path = format!("{system_path}\\System32\\dinput8.dll");

        let lib =
            unsafe { libloading::Library::new(&real_path) }.expect("Failed to load real dinput8");

        unsafe {
            REAL_DIRECTINPUT8CREATE = Some(
                *lib.get::<unsafe extern "system" fn(
                    HINSTANCE,
                    u32,
                    GUID,
                    *mut *mut c_void,
                    *mut c_void,
                ) -> HRESULT>(b"DirectInput8Create\0")
                    .unwrap(),
            );
            std::mem::forget(lib); // Don't drop it, keep loaded
        }
    });
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    _hinst_dll: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: *mut std::os::raw::c_void,
) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;
    const DLL_PROCESS_DETACH: u32 = 0;
    const DLL_THREAD_ATTACH: u32 = 2;
    const DLL_THREAD_DETACH: u32 = 3;

    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            load_real_dinput8();
            let game = Game::get_current_game();
            match game {
                Game::DMCLauncher | Game::Unknown => {}
                Game::DMC1 | Game::DMC2 | Game::DMC3 => {
                    create_console();
                    randomizer_utilities::setup_logger("dmc_mod_loader");
                    match game.get_current_version() {
                        Ok(version_info) => {
                            if !version_info.valid_for_use {
                                determine_error_message(&version_info);
                            }
                            let mod_list = game.identify_mods();
                            load_mods(&mod_list);
                            let status = LoaderStatus {
                                game_information: version_info,
                                mod_information: mod_list,
                            };
                            STATUS.set(status).unwrap();
                            let name = CString::new(format!("{game}_randomizer.dll"));
                            if let Err(err) = unsafe {
                                LoadLibraryA(PCSTR::from_raw(name.unwrap().as_ptr() as *const u8))
                            } {
                                log::error!("Failed to load randomizer for {}: {}", game, err);
                                log::warn!(
                                    "This is safe to ignore if the relevant randomizer is not installed or does not exist"
                                )
                            }
                        }
                        Err(err) => {
                            log::error!("{}", err);
                        }
                    }
                }
            }
        }
        DLL_PROCESS_DETACH => {
            // For cleanup
        }
        DLL_THREAD_ATTACH | DLL_THREAD_DETACH => {
            // Normally ignored if DisableThreadLibraryCalls is used
        }
        _ => {}
    }
    BOOL(1)
}

fn load_mods(mod_list: &[VersionInformation]) {
    for game_mod in mod_list {
        log::debug!("Attempting to load: {}", game_mod);
        if should_load(game_mod) {
            log::debug!(
                "Loading mod: {}",
                game_mod.mod_type.unwrap().get_file_name()
            );
            let name = CString::new(game_mod.mod_type.unwrap().get_file_name());
            if let Err(err) =
                unsafe { LoadLibraryA(PCSTR::from_raw(name.unwrap().as_ptr() as *const u8)) }
            {
                log::error!("Failed to load mod: {}", err);
            }
        }
    }
}

fn should_load(version_info: &VersionInformation) -> bool {
    if version_info.valid_for_use {
        if let Some(mod_type) = version_info.mod_type {
            return match mod_type {
                Mod::Eva | Mod::Lucia | Mod::Mary => !config::CONFIG.mods.disable_ddmk,
                Mod::Crimson => !config::CONFIG.mods.disable_crimson,
            };
        }
    } else {
        log::error!("{}", determine_error_message(version_info));
        return false;
    }
    false
}

fn determine_error_message(ver_info: &VersionInformation) -> String {
    match ver_info.mod_type {
        None => {
            format!(
                "{} is not suitable for randomizer use, please re-patch",
                ver_info.description
            )
        }
        Some(mod_type) => match mod_type {
            Mod::Eva | Mod::Lucia | Mod::Mary => format!(
                "Current DDMK Version: ({}) is not 2.7.3",
                ver_info.description
            ),
            Mod::Crimson => format!(
                "Current Crimson Version: ({}) is not 0.4",
                ver_info.description
            ),
        },
    }
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DirectInput8Create(
    hinst: HINSTANCE,
    dwVersion: u32,
    riidltf: GUID,
    ppvOut: *mut *mut c_void,
    punkOuter: *mut c_void,
) -> HRESULT {
    unsafe {
        // call into the real dinput8.dll
        load_real_dinput8(); // lazy-load if needed
        REAL_DIRECTINPUT8CREATE.expect("not loaded")(hinst, dwVersion, riidltf, ppvOut, punkOuter)
    }
}

pub fn create_console() {
    unsafe {
        if AllocConsole().is_ok() {
            pub fn enable_ansi_support() -> Result<(), anyhow::Error> {
                // So we can have sweet sweet color
                unsafe {
                    let handle = GetStdHandle(STD_OUTPUT_HANDLE)?;
                    if handle == HANDLE::default() {
                        return Err(anyhow!(windows::core::Error::from(GetLastError())));
                    }

                    let mut mode = std::mem::zeroed();
                    GetConsoleMode(handle, &mut mode)?;
                    SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING)?;
                    Ok(())
                }
            }
            if let Err(err) = enable_ansi_support() {
                log::error!("Failed to enable ANSI support: {}", err);
            }
            log::info!("Console created successfully!");
        } else {
            log::info!("Failed to allocate console!");
        }
    }
}
