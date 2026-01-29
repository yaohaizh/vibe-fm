//! Windows Shell Context Menu implementation
//!
//! This module provides functionality to show the native Windows Explorer
//! context menu for files and folders.

use std::path::{Path, PathBuf};

/// Show the native Windows Explorer context menu for the given paths.
/// Uses the current cursor position for the menu location.
#[cfg(target_os = "windows")]
pub fn show_context_menu_for_paths(paths: &[PathBuf]) -> Result<bool, String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HWND, POINT};
    use windows::Win32::System::Com::{
        CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{
        BHID_SFUIObject, IContextMenu, IShellItemArray, SHCreateShellItemArrayFromIDLists,
        SHParseDisplayName, CMF_NORMAL, CMINVOKECOMMANDINFO,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreatePopupMenu, DestroyMenu, GetCursorPos, TrackPopupMenu, SW_SHOWNORMAL, TPM_LEFTALIGN,
        TPM_RETURNCMD, TPM_RIGHTBUTTON, TPM_TOPALIGN,
    };

    if paths.is_empty() {
        return Err("No paths provided".to_string());
    }

    // Initialize COM
    let com_result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    if com_result.is_err() && com_result != windows::Win32::Foundation::CO_E_ALREADYINITIALIZED {
        return Err(format!("Failed to initialize COM: {:?}", com_result));
    }

    let result = (|| -> Result<bool, String> {
        // Get current cursor position
        let mut cursor_pos = POINT::default();
        unsafe {
            if GetCursorPos(&mut cursor_pos).is_err() {
                return Err("Failed to get cursor position".to_string());
            }
        }

        // Convert paths to PIDLs
        let mut pidls: Vec<*mut windows::Win32::UI::Shell::Common::ITEMIDLIST> = Vec::new();

        for path in paths {
            let path_str = path.to_string_lossy();
            let wide: Vec<u16> = OsStr::new(path_str.as_ref())
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut pidl = std::ptr::null_mut();
            let result =
                unsafe { SHParseDisplayName(PCWSTR(wide.as_ptr()), None, &mut pidl, 0, None) };

            if result.is_err() {
                // Clean up already allocated PIDLs
                for p in &pidls {
                    unsafe {
                        CoTaskMemFree(Some(*p as *const _));
                    }
                }
                return Err(format!("Failed to parse path: {:?}", path));
            }

            pidls.push(pidl);
        }

        // Create shell item array from PIDLs
        let pidl_ptrs: Vec<*const windows::Win32::UI::Shell::Common::ITEMIDLIST> =
            pidls.iter().map(|p| *p as *const _).collect();

        let shell_item_array: IShellItemArray =
            match unsafe { SHCreateShellItemArrayFromIDLists(&pidl_ptrs) } {
                Ok(sia) => sia,
                Err(e) => {
                    // Clean up PIDLs
                    for p in &pidls {
                        unsafe {
                            CoTaskMemFree(Some(*p as *const _));
                        }
                    }
                    return Err(format!("Failed to create shell item array: {:?}", e));
                }
            };

        // Clean up PIDLs
        for p in &pidls {
            unsafe {
                CoTaskMemFree(Some(*p as *const _));
            }
        }

        // Get the IContextMenu interface
        let context_menu: IContextMenu = match unsafe {
            shell_item_array.BindToHandler::<_, IContextMenu>(None, &BHID_SFUIObject)
        } {
            Ok(cm) => cm,
            Err(e) => {
                return Err(format!("Failed to get context menu: {:?}", e));
            }
        };

        // Create popup menu
        let hmenu = unsafe { CreatePopupMenu() }
            .map_err(|e| format!("Failed to create popup menu: {:?}", e))?;

        // Query context menu items
        let query_result =
            unsafe { context_menu.QueryContextMenu(hmenu, 0, 1, 0x7FFF, CMF_NORMAL) };

        if query_result.is_err() {
            unsafe {
                let _ = DestroyMenu(hmenu);
            }
            return Err(format!("Failed to query context menu: {:?}", query_result));
        }

        // Show the popup menu
        let cmd = unsafe {
            TrackPopupMenu(
                hmenu,
                TPM_LEFTALIGN | TPM_TOPALIGN | TPM_RETURNCMD | TPM_RIGHTBUTTON,
                cursor_pos.x,
                cursor_pos.y,
                0,
                HWND::default(),
                None,
            )
        };

        if cmd.0 > 0 {
            // User selected a command - invoke it
            let mut invoke_info: CMINVOKECOMMANDINFO = unsafe { std::mem::zeroed() };
            invoke_info.cbSize = std::mem::size_of::<CMINVOKECOMMANDINFO>() as u32;
            invoke_info.hwnd = HWND::default();
            invoke_info.lpVerb = windows::core::PCSTR((cmd.0 - 1) as *const u8);
            invoke_info.nShow = SW_SHOWNORMAL.0;

            let invoke_result = unsafe { context_menu.InvokeCommand(&invoke_info) };

            unsafe {
                let _ = DestroyMenu(hmenu);
            }

            if invoke_result.is_err() {
                return Err(format!("Failed to invoke command: {:?}", invoke_result));
            }

            Ok(true)
        } else {
            // User cancelled the menu
            unsafe {
                let _ = DestroyMenu(hmenu);
            }
            Ok(false)
        }
    })();

    // Uninitialize COM
    unsafe {
        CoUninitialize();
    }

    result
}

#[cfg(not(target_os = "windows"))]
pub fn show_context_menu_for_paths(_paths: &[PathBuf]) -> Result<bool, String> {
    // On non-Windows platforms, this is a no-op
    Err("Native context menu is only supported on Windows".to_string())
}

/// Show context menu for a single path (convenience function)
pub fn show_context_menu_for_path(path: &Path) -> Result<bool, String> {
    show_context_menu_for_paths(&[path.to_path_buf()])
}
