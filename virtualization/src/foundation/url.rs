use virtualization_sys as vz_sys;
use vz_sys::INSURL;

use super::NSString;

pub struct NSURL(vz_sys::NSURL);
impl NSURL {
    pub fn new(path: &str) -> Self {
        let str = NSString::new(path);
        let inner = unsafe { vz_sys::NSURL::fileURLWithPath_(str.into_inner()) };
        Self(inner)
    }

    pub fn into_inner(self) -> vz_sys::NSURL {
        self.0
    }
}
