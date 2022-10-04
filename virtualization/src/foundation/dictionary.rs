use std::fmt;

use virtualization_sys as vz_sys;
use vz_sys::{INSDictionary, INSEnumerator};

use super::{NSError, NSString};

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

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        let length = unsafe {
            <vz_sys::NSDictionary as INSDictionary<vz_sys::id, vz_sys::id>>::count(&self.inner)
        };
        length as usize
    }
}

impl Default for NSDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for NSDictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let copy = NSDictionary::from(self.inner.0);
        let mut map = f.debug_map();
        for (k, v) in copy {
            let value_class = unsafe { (*v).class() };
            let value_str = match value_class.name() {
                "__NSCFString" => {
                    let str = NSString::from(v);
                    String::from(str.as_str())
                }
                "NSError" => {
                    let err = NSError::from(v);
                    format!("{:?}", err)
                }
                _ => {
                    format!("{:?} ({})", v, value_class.name())
                }
            };
            map.entry(&k, &value_str);
        }
        map.finish()
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
    type Item = (NSString, vz_sys::id);

    fn next(&mut self) -> Option<Self::Item> {
        let key_ptr = unsafe {
            <vz_sys::NSEnumerator as INSEnumerator<vz_sys::id>>::nextObject(&self.key_enum)
        };
        if !key_ptr.is_null() {
            let value_ptr = unsafe {
                <vz_sys::NSDictionary as INSDictionary<vz_sys::id, vz_sys::id>>::objectForKey_(
                    &self.inner,
                    key_ptr,
                )
            };
            let key = NSString::from(key_ptr);
            let value = value_ptr;
            Some((key, value))
        } else {
            None
        }
    }
}
