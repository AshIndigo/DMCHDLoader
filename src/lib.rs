mod config;
mod status;
mod utilities;

use crate::status::Status;
use crate::utilities::is_ddmk_loaded;
use anyhow::anyhow;
use std::env::current_exe;
use std::ffi::c_void;
use std::fs;
use std::io::ErrorKind;
use windows::core::{BOOL, GUID, HRESULT, PCSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::{
    AllocConsole, GetConsoleMode, GetStdHandle, SetConsoleMode,
    ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
};
use windows::Win32::System::LibraryLoader::LoadLibraryA;
use xxhash_rust::const_xxh3::xxh3_64;

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
    const DMC_LAUNCHER: &str = "dmcLauncher.exe";
    const DMC1: &str = "dmc1.exe";
    const DMC2: &str = "dmc2.exe";
    const DMC3: &str = "dmc3.exe";

    match fdw_reason {
        DLL_PROCESS_ATTACH => {
            load_real_dinput8();
            let mut status: Status = Status {
                crimson_hash_error: false,
                dmc3_hash_error: false,
            };
            match current_exe()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
            {
                DMC_LAUNCHER => {
                    // Do nothing but is here in case I ever want to do something to the launcher
                }
                DMC1 => {
                    create_console();
                    randomizer_utilities::setup_logger("dmc_mod_loader");
                    match load_dmc1_mods(&mut status) {
                        Ok(_) => {
                            log::info!("Successfully loaded other dlls");
                        }
                        Err(err) => {
                            log::error!("Failed to load other dlls: {}", err);
                        }
                    }
                    match is_file_valid("dmc1.exe", 16596094990179088469) {
                        Ok(_) => {
                            log::info!("Valid install of DMC1 detected!");
                        }
                        Err(err) => match err.kind() {
                            ErrorKind::InvalidData => {
                                log::error!(
                                    "DMC1 does not match the expected hash, bad things may occur! Please downgrade/repatch your game."
                                );
                                status.dmc3_hash_error = true;
                            }
                            ErrorKind::NotFound => {
                                log::error!(
                                    "DMC1 does not exist! How in the world did you manage this"
                                );
                            }
                            _ => {
                                log::error!("Unexpected error: {}", err);
                            }
                        },
                    }
                    let _ = unsafe {
                        LoadLibraryA(PCSTR::from_raw(c"dmc1_randomizer.dll".as_ptr() as *const u8))
                    };
                }
                DMC2 => {
                    // Do nothing
                }
                DMC3 => {
                    create_console();
                    randomizer_utilities::setup_logger("dmc_mod_loader");
                    match load_dmc3_mods(&mut status) {
                        Ok(_) => {
                            log::info!("Successfully loaded other dlls");
                        }
                        Err(err) => {
                            log::error!("Failed to load other dlls: {}", err);
                        }
                    }
                    match is_file_valid("dmc3.exe", 9031715114876197692) {
                        Ok(_) => {
                            log::info!("Valid install of DMC3 detected!");
                        }
                        Err(err) => match err.kind() {
                            ErrorKind::InvalidData => {
                                log::error!(
                                    "DMC3 does not match the expected hash, bad things may occur! Please downgrade/repatch your game."
                                );
                                status.dmc3_hash_error = true;
                            }
                            ErrorKind::NotFound => {
                                log::error!(
                                    "DMC3 does not exist! How in the world did you manage this"
                                );
                            }
                            _ => {
                                log::error!("Unexpected error: {}", err);
                            }
                        },
                    }
                    let _ = unsafe {
                        LoadLibraryA(PCSTR::from_raw(c"dmc3_randomizer.dll".as_ptr() as *const u8))
                    };
                }
                _ => {}
            }
            status::STATUS.set(status).expect("Unable to set status");
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

fn load_dmc1_mods(status: &mut Status) -> Result<(), std::io::Error> {
    if !config::CONFIG.mods.disable_ddmk {
        match is_file_valid("Eva.dll", 2536699235936189826) {
            Ok(_) => {
                let _ = unsafe { LoadLibraryA(PCSTR::from_raw(c"Eva.dll".as_ptr() as *const u8)) };
            }
            Err(err) => match err.kind() {
                ErrorKind::InvalidData => {
                    log::error!("Eva/DDMK Hash does not match version 2.7.3, please update DDMK");
                }
                ErrorKind::NotFound => {}
                _ => {
                    log::error!("Unexpected error: {}", err);
                }
            },
        }
    }
    Ok(())
}

fn load_dmc3_mods(status: &mut Status) -> Result<(), std::io::Error> {
    // The game will immolate if both of these try to load
    if !config::CONFIG.mods.disable_ddmk {
        match is_file_valid("Mary.dll", 7087074874482460961) {
            Ok(_) => {
                let _ = unsafe { LoadLibraryA(PCSTR::from_raw(c"Mary.dll".as_ptr() as *const u8)) };
            }
            Err(err) => match err.kind() {
                ErrorKind::InvalidData => {
                    log::error!("Mary/DDMK Hash does not match version 2.7.3, please update DDMK");
                }
                ErrorKind::NotFound => {}
                _ => {
                    log::error!("Unexpected error: {}", err);
                }
            },
        }
    }
    if !config::CONFIG.mods.disable_crimson && !is_ddmk_loaded() {
        match is_file_valid("Crimson.dll", 6027093939875741571) {
            Ok(_) => {}
            Err(err) => match err.kind() {
                ErrorKind::InvalidData => {
                    log::error!("Crimson Hash does not match version 0.4");
                    status.crimson_hash_error = true;
                }
                ErrorKind::NotFound => {}
                _ => {
                    log::error!("Unexpected error: {}", err);
                }
            },
        }
        let _ = unsafe { LoadLibraryA(PCSTR::from_raw(c"Crimson.dll".as_ptr() as *const u8)) };
    }
    Ok(())
}

fn is_file_valid(file_path: &str, expected_hash: u64) -> Result<(), std::io::Error> {
    let data = fs::read(file_path)?;
    //log::debug!("Hash for {file_path}: {}", xxh3_64(&data));
    if xxh3_64(&data) == expected_hash {
        Ok(())
    } else {
        Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "File has invalid hash",
        ))
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
            match enable_ansi_support() {
                Ok(_) => {}
                Err(err) => {
                    log::error!("Failed to enable ANSI support: {}", err);
                }
            }
            log::info!("Console created successfully!");
        } else {
            log::info!("Failed to allocate console!");
        }
    }
}
