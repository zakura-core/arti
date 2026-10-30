//! Query the memory available to Tor without enumerating processes or CPUs.

/// Return the system's physical RAM, capped by the existing Linux cgroup limits.
///
/// A failed physical-memory query returns `None`, retaining the configuration's
/// existing fallback. A cgroup can legitimately report a zero-byte limit.
pub(super) fn available_memory() -> Option<u64> {
    cfg_if::cfg_if! {
        if #[cfg(any(target_os = "linux", target_os = "android"))] {
            let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;
            let v2 = std::fs::read_to_string("/sys/fs/cgroup/memory.max").ok();
            let v1 = std::fs::read_to_string(
                "/sys/fs/cgroup/memory/memory.limit_in_bytes",
            ).ok();
            linux_memory(&meminfo, v2.as_deref(), v1.as_deref())
        } else if #[cfg(any(target_os = "macos", target_os = "ios"))] {
            sysctl_memory(c"hw.memsize").filter(|bytes| *bytes != 0)
        } else if #[cfg(target_os = "freebsd")] {
            // Match the previous provider: prefer usable pages to hw.physmem,
            // which also includes memory reserved by the kernel.
            sysctl_memory(c"vm.stats.vm.v_page_count")
                .zip(page_size())
                .map(|(pages, size)| pages.saturating_mul(size))
                .or_else(|| sysctl_memory(c"hw.physmem"))
                .filter(|bytes| *bytes != 0)
        } else if #[cfg(target_os = "netbsd")] {
            // NetBSD's sysconf page count queries the same uvmexp information
            // used by the previous provider.
            // SAFETY: sysconf takes no pointers and these selectors are valid.
            let pages = unsafe { libc::sysconf(libc::_SC_PHYS_PAGES) };
            u64::try_from(pages).ok()
                .zip(page_size())
                .map(|(pages, size)| pages.saturating_mul(size))
                .filter(|bytes| *bytes != 0)
        } else if #[cfg(windows)] {
            windows_memory()
        } else {
            None
        }
    }
}

/// Read a four- or eight-byte unsigned sysctl value without assuming word size.
#[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
fn sysctl_memory(name: &std::ffi::CStr) -> Option<u64> {
    let mut bytes = [0_u8; 8];
    let mut len = bytes.len();
    // SAFETY: name is NUL-terminated. The output buffer has exactly `len`
    // writable bytes; a null new-value pointer makes this a read-only query.
    let result = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            bytes.as_mut_ptr().cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if result != 0 {
        return None;
    }
    match len {
        4 => Some(u64::from(u32::from_ne_bytes(bytes[..4].try_into().ok()?))),
        8 => Some(u64::from_ne_bytes(bytes)),
        _ => None,
    }
}

/// Read the native page size, rejecting the error sentinel from sysconf.
#[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
fn page_size() -> Option<u64> {
    // SAFETY: sysconf takes no pointers and this selector is valid.
    let size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    u64::try_from(size).ok().filter(|size| *size != 0)
}

/// Query physical memory through the Windows system-information API.
#[cfg(windows)]
fn windows_memory() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    // SAFETY: MEMORYSTATUSEX contains only integers; zero is valid for them all.
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = u32::try_from(std::mem::size_of::<MEMORYSTATUSEX>()).ok()?;
    // SAFETY: status is initialized, has the required size field, and remains
    // writable for the duration of the call.
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return None;
    }
    Some(status.ullTotalPhys).filter(|bytes| *bytes != 0)
}

/// Parse Linux's KiB-valued physical RAM and byte-valued cgroup memory ceilings.
#[cfg(any(target_os = "linux", target_os = "android", test))]
fn linux_memory(meminfo: &str, v2: Option<&str>, v1: Option<&str>) -> Option<u64> {
    const BYTES_PER_KIB: u64 = 1024;
    let physical = meminfo.lines().find_map(|line| {
        let value = line.strip_prefix("MemTotal:")?;
        let mut words = value.split_ascii_whitespace();
        let kib = words.next()?.parse::<u64>().ok()?;
        (words.next()? == "kB").then_some(kib.saturating_mul(BYTES_PER_KIB))
    })?;
    if physical == 0 {
        return None;
    }
    // Like the previous provider, "max", absent files and malformed limits do
    // not impose a ceiling. Reading the limit does not require usage counters.
    let limit = v2
        .into_iter()
        .chain(v1)
        .filter_map(|value| value.trim().parse::<u64>().ok())
        .min();
    Some(limit.map_or(physical, |limit| physical.min(limit)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn linux_limits_and_units() {
        let meminfo = "MemFree: 1 kB\nMemTotal: 8192 kB\n";
        let physical = 8 * 1024 * 1024;
        assert_eq!(linux_memory(meminfo, None, None), Some(physical));
        assert_eq!(linux_memory(meminfo, Some("max\n"), None), Some(physical));
        assert_eq!(
            linux_memory(meminfo, Some("4194304\n"), None),
            Some(4194304)
        );
        assert_eq!(
            linux_memory(meminfo, None, Some("2097152\n")),
            Some(2097152)
        );
        assert_eq!(
            linux_memory(meminfo, Some("4194304"), Some("2097152")),
            Some(2097152)
        );
        assert_eq!(
            linux_memory(meminfo, Some("16777216"), None),
            Some(physical)
        );
        assert_eq!(linux_memory(meminfo, Some("0"), None), Some(0));
        assert_eq!(linux_memory(meminfo, Some("invalid"), None), Some(physical));
    }

    #[test]
    fn linux_query_failures_and_saturation() {
        for malformed in [
            "",
            "MemFree: 1024 kB",
            "MemTotal: 0 kB",
            "MemTotal: -1 kB",
            "MemTotal: 1 MB",
        ] {
            assert_eq!(linux_memory(malformed, Some("1024"), None), None);
        }
        assert_eq!(
            linux_memory("MemTotal: 18446744073709551615 kB", None, None),
            Some(u64::MAX),
        );
        assert_eq!(
            linux_memory("MemTotal: 18446744073709551615 kB", Some("1024"), None),
            Some(1024),
        );
    }

    #[test]
    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "linux",
        target_os = "android",
        target_os = "freebsd",
        target_os = "netbsd",
        windows
    ))]
    fn native_query_matches_previous_provider() {
        use sysinfo::{MemoryRefreshKind, System};
        let mut previous = System::new();
        previous.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        let physical = previous.total_memory();
        assert_ne!(physical, 0);
        let previous = previous
            .cgroup_limits()
            .map_or(physical, |limit| physical.min(limit.total_memory));
        let actual = available_memory().expect("native memory query");
        // Limit files alone can cap memory even when the old provider cannot
        // read the unrelated usage/stat counters it also requires.
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            let meminfo =
                std::fs::read_to_string("/proc/meminfo").expect("proc memory information");
            assert_eq!(linux_memory(&meminfo, None, None), Some(physical));
            assert!(actual <= previous);
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        assert_eq!(actual, previous);
    }
}
