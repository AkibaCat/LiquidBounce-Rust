// SPDX-License-Identifier: GPL-3.0-or-later
//! Initial system FORMAT locale, separately from the game's resource language.

/// The process's initial user-region locale, as used by the JVM default FORMAT
/// locale. Keep the OS boundary in the platform crate; the client calls safely.
pub fn initial_format_locale() -> &'static str {
    static LOCALE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    LOCALE.get_or_init(|| {
        #[cfg(target_os = "windows")]
        {
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn GetUserDefaultLocaleName(name: *mut u16, count: i32) -> i32;
            }
            let mut name = [0_u16; 85];
            // SAFETY: the API receives a valid writable UTF-16 array and its
            // full capacity. Its documented count includes the terminating NUL.
            let count = unsafe { GetUserDefaultLocaleName(name.as_mut_ptr(), name.len() as i32) };
            if count > 1 && count as usize <= name.len() {
                return String::from_utf16_lossy(&name[..count as usize - 1]);
            }
        }
        for key in ["LC_ALL", "LC_NUMERIC", "LANG"] {
            if let Ok(value) = std::env::var(key)
                && !value.is_empty()
            {
                return value.split('.').next().unwrap_or("").replace('_', "-");
            }
        }
        "und".to_owned()
    })
}
