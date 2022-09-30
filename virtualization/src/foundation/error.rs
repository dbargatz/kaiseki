use std::fmt;

use virtualization_sys as vz_sys;
use vz_sys::INSError;

use super::{NSDictionary, NSString};

pub struct NSError(vz_sys::NSError);
impl NSError {
    pub fn new() -> Self {
        let inner = vz_sys::NSError::alloc();
        Self(inner)
    }

    pub fn code(&self) -> usize {
        unsafe { self.0.code() as usize }
    }

    pub fn into_inner(self) -> vz_sys::NSError {
        self.0
    }

    pub fn localized_description(&self) -> String {
        let str = unsafe { self.0.localizedDescription() };
        let inner = NSString::from(str);
        String::from(inner.as_str())
    }

    pub fn user_info(&self) -> NSDictionary {
        let inner_dict = unsafe { self.0.userInfo() };
        NSDictionary::from(inner_dict)
    }
}

impl fmt::Display for NSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("NSError:\n")?;
        f.write_str(format!("  code       : {}\n", self.code()).as_str())?;
        f.write_str(format!("  description: {}\n", self.localized_description()).as_str())?;
        f.write_str("  userinfo   : {")?;
        for (k, v) in self.user_info() {
            f.write_str(format!("\n    {}       : {}", k, v).as_str())?;
        }
        f.write_str("  }")
    }
}

impl fmt::Debug for NSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NSError")
            .field("code", &self.code())
            .field("description", &self.localized_description())
            .finish()
    }
}

impl From<vz_sys::NSError> for NSError {
    fn from(p: vz_sys::NSError) -> Self {
        NSError(p)
    }
}
