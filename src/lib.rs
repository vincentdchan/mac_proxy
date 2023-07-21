use std::ffi::{c_char, c_long, c_void};
use core_foundation::dictionary::*;
use core_foundation::string::*;
use core_foundation::array::*;
use core_foundation::base::{CFGetTypeID, CFRelease};
use core_foundation::number::{kCFNumberLongType, CFNumberGetTypeID, CFNumberGetValue};
use serde_json::Value;

#[allow(dead_code)]
pub type CFDictionaryRef = *const __CFDictionary;
#[allow(dead_code)]
pub type CFStringRef = *const __CFString;

#[link(name = "CFNetwork", kind = "framework")]
extern "C" {
    pub fn CFNetworkCopySystemProxySettings() -> CFDictionaryRef;

}

fn dict_iterator(key: *const c_void, value: *const c_void, context: *mut c_void) {
    unsafe {
        let result_map = context.cast::<serde_json::Map<String, Value>>();
        let key_str: CFStringRef = key.cast();
        let value_type = CFGetTypeID(value.cast());

        let mut key_cstr: [c_char; 256] = [0; 256];
        CFStringGetCString(key_str, key_cstr.as_mut_ptr(), 256, kCFStringEncodingUTF8);

        if value_type == CFStringGetTypeID() {
            let value_str: CFStringRef = value.cast();
            let mut value_cstr: [c_char; 256] = [0; 256];

            CFStringGetCString(
                value_str,
                value_cstr.as_mut_ptr(),
                256,
                kCFStringEncodingUTF8,
            );

            let key_string = std::ffi::CStr::from_ptr(key_cstr.as_ptr())
                .to_str()
                .unwrap()
                .to_string();
            let value_string = std::ffi::CStr::from_ptr(value_cstr.as_ptr())
                .to_str()
                .unwrap()
                .to_string();

            (*result_map).insert(key_string, Value::String(value_string));
        } else if value_type == CFNumberGetTypeID() {
            let mut stack_value: c_long = 0;
            let value_ptr = &mut stack_value as *mut c_long;
            CFNumberGetValue(value.cast(), kCFNumberLongType, value_ptr.cast());

            let key_string = std::ffi::CStr::from_ptr(key_cstr.as_ptr())
                .to_str()
                .unwrap()
                .to_string();

            (*result_map).insert(key_string, Value::Number(stack_value.into()));
        } else if value_type == CFDictionaryGetTypeID() {
            let value_dict: CFDictionaryRef = value.cast();
            let value_map = serde_json::Map::new();

            let iterator_fn: CFDictionaryApplierFunction =
                std::mem::transmute(dict_iterator as fn(*const c_void, *const c_void, *mut c_void));
            let value_map_raw = Box::into_raw(Box::new(value_map));
            CFDictionaryApplyFunction(value_dict, iterator_fn, value_map_raw.cast());

            let key_string = std::ffi::CStr::from_ptr(key_cstr.as_ptr())
                .to_str()
                .unwrap()
                .to_string();

            (*result_map).insert(key_string, Value::Object(*Box::from_raw(value_map_raw)));
        } else if value_type == CFArrayGetTypeID() {
            let value_array: CFArrayRef = value.cast();
            let mut value_vec = Vec::new();

            let count = CFArrayGetCount(value_array);
            for i in 0..count {
                let value = CFArrayGetValueAtIndex(value_array, i);
                let value_type = CFGetTypeID(value.cast());

                if value_type == CFStringGetTypeID() {
                    let value_str: CFStringRef = value.cast();
                    let mut value_cstr: [c_char; 256] = [0; 256];

                    CFStringGetCString(
                        value_str,
                        value_cstr.as_mut_ptr(),
                        256,
                        kCFStringEncodingUTF8,
                    );

                    let value_string = std::ffi::CStr::from_ptr(value_cstr.as_ptr())
                        .to_str()
                        .unwrap()
                        .to_string();
                    value_vec.push(Value::String(value_string));
                } else if value_type == CFNumberGetTypeID() {
                    let mut stack_value: c_long = 0;
                    let value_ptr = &mut stack_value as *mut c_long;
                    CFNumberGetValue(value.cast(), kCFNumberLongType, value_ptr.cast());

                    value_vec.push(Value::Number(stack_value.into()));
                }
            }

            let key_string = std::ffi::CStr::from_ptr(key_cstr.as_ptr())
                .to_str()
                .unwrap()
                .to_string();

            (*result_map).insert(key_string, Value::Array(value_vec));
        }
    }
}

#[cfg(target_os = "macos")]
pub fn mac_proxy_settings() -> Option<serde_json::Map<String, Value>> {
    unsafe {
        let dict_ref = CFNetworkCopySystemProxySettings();
        if dict_ref.is_null() {
            return None;
        }
        let result = Box::new(serde_json::Map::new());

        let iterator_fn: CFDictionaryApplierFunction =
            std::mem::transmute(dict_iterator as fn(*const c_void, *const c_void, *mut c_void));
        let result_raw = Box::into_raw(result);
        CFDictionaryApplyFunction(dict_ref, iterator_fn, result_raw.cast());

        CFRelease(dict_ref.cast::<c_void>());
        Some(*Box::from_raw(result_raw))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn mac_proxy_settings() -> Option<serde_json::Map<String, Value>> {
    return None;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_system_proxy_settings() {
        let settings = super::mac_proxy_settings();
        let json = serde_json::to_string_pretty(&settings.unwrap()).unwrap();
        println!("settings: {}", json);
    }
}
