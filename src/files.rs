use glib_sys;
use libc::c_void;
use std::ffi::CStr;

pub fn compute_md5(data: &[u8]) -> String {
    let ret;
    unsafe {
        let md5 = glib_sys::g_compute_checksum_for_data(
            glib_sys::G_CHECKSUM_MD5,
            data.as_ptr() as *mut u8,
            data.len(),
        );
        ret = CStr::from_ptr(md5).to_string_lossy();

        glib_sys::g_free(md5 as *mut c_void);
    }

    return ret.into_owned();
}
