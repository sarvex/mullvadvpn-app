use std::ffi::CStr;
/// name of the group that should be excluded
const EXCLUSION_GROUP: &[u8] = b"mullvad-exclusion\0";

/// Returns the GID of `mullvad-exclusion` group if it exists.
pub fn get_exclusion_gid() -> Option<u32> {
    let exclusion_group_name = unsafe { CStr::from_ptr(EXCLUSION_GROUP.as_ptr() as *const i8) };
    talpid_core::macos::get_group_id(exclusion_group_name)
}


#[cfg(test)]
mod test {
    #[test]
    fn test_exclusion_gid() {
        let _ = super::get_exclusion_gid();
    }
}
