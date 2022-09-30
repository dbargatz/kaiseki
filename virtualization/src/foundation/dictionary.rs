use virtualization_sys as vz_sys;
use vz_sys::{INSDictionary, INSEnumerator};

use super::NSString;

pub struct NSDictionary {
    inner: vz_sys::NSDictionary,
    key_enum: vz_sys::NSEnumerator,
}

impl NSDictionary {
    pub fn new() -> Self {
        let inner = vz_sys::NSDictionary::alloc();
        let inner = unsafe {
            let ptr = <vz_sys::NSDictionary as INSDictionary<vz_sys::id, vz_sys::id>>::init(&inner);
            vz_sys::NSDictionary(ptr)
        };
        let key_enum = unsafe {
            <vz_sys::NSDictionary as INSDictionary<vz_sys::id, vz_sys::id>>::keyEnumerator(&inner)
        };
        Self { inner, key_enum }
    }

    pub fn into_inner(self) -> vz_sys::NSDictionary {
        self.inner
    }
}

impl From<vz_sys::id> for NSDictionary {
    fn from(p: vz_sys::id) -> Self {
        NSDictionary::from(vz_sys::NSDictionary(p))
    }
}

impl From<vz_sys::NSDictionary> for NSDictionary {
    fn from(p: vz_sys::NSDictionary) -> Self {
        let key_enum = unsafe {
            <vz_sys::NSDictionary as INSDictionary<vz_sys::id, vz_sys::id>>::keyEnumerator(&p)
        };
        NSDictionary { inner: p, key_enum }
    }
}

impl Iterator for NSDictionary {
    type Item = (NSString, NSString);

    fn next(&mut self) -> Option<Self::Item> {
        let key_ptr = unsafe {
            <vz_sys::NSEnumerator as INSEnumerator<vz_sys::id>>::nextObject(&self.key_enum)
        };
        if std::ptr::null_mut() != key_ptr {
            let value_ptr = unsafe {
                <vz_sys::NSDictionary as INSDictionary<vz_sys::id, vz_sys::id>>::objectForKey_(
                    &self.inner,
                    key_ptr,
                )
            };
            let key = NSString::from(key_ptr);
            let value = NSString::from(value_ptr);
            Some((key, value))
        } else {
            None
        }
    }
}
