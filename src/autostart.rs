use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Ticky";

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(Some(0)).collect()
}

fn exe_command() -> Result<String, String> {
    let path = std::env::current_exe().map_err(|err| err.to_string())?;
    let quoted = format!("\"{}\"", path.to_string_lossy());
    Ok(quoted)
}

fn open_key(write: bool) -> Result<HKEY, String> {
    let subkey = wide(RUN_KEY);
    let mut handle = HKEY::default();
    let access = if write { KEY_WRITE } else { KEY_READ };
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR::from_raw(subkey.as_ptr()),
            None,
            access,
            &mut handle,
        )
    };
    result.ok().map_err(|err| err.to_string())?;
    Ok(handle)
}

pub fn is_enabled() -> Result<bool, String> {
    let handle = open_key(false)?;
    let name = wide(VALUE_NAME);
    let mut length = 0u32;

    let probe = unsafe {
        RegQueryValueExW(
            handle,
            PCWSTR::from_raw(name.as_ptr()),
            None,
            None,
            None,
            Some(&mut length),
        )
    };
    unsafe { let _ = RegCloseKey(handle); };

    if probe == ERROR_FILE_NOT_FOUND {
        return Ok(false);
    }
    if probe != ERROR_SUCCESS {
        return Err(format!("could not read startup setting: {probe:?}"));
    }
    Ok(length > 2)
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let handle = open_key(true)?;
    let name = wide(VALUE_NAME);

    let result = if enabled {
        let command = exe_command()?;
        let value = wide(&command);
        let bytes = unsafe {
            std::slice::from_raw_parts(value.as_ptr() as *const u8, value.len() * 2)
        };
        unsafe {
            RegSetValueExW(
                handle,
                PCWSTR::from_raw(name.as_ptr()),
                None,
                REG_SZ,
                Some(bytes),
            )
        }
    } else {
        unsafe { RegDeleteValueW(handle, PCWSTR::from_raw(name.as_ptr())) }
    };

    unsafe { let _ = RegCloseKey(handle); };

    if result == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }
    result.ok().map_err(|err| err.to_string())
}
