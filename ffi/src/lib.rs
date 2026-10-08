//! Stable C ABI for the RVA core.
//!
//! This is the single native bridge used by Swift, Kotlin, Flutter (and C)
//! adapters. It exposes the same contract as the WASM boundary
//! (see `adapters/contract.json`):
//!
//! ```c
//! RvaHandle *rva_open(const uint8_t *data, size_t len);
//! char      *rva_describe(const RvaHandle *handle);
//! char      *rva_resolve(const RvaHandle *handle, uint32_t width, uint32_t height);
//! uint8_t   *rva_resource(const RvaHandle *handle, const char *name, size_t *out_len);
//! bool       rva_has_resource(const RvaHandle *handle, const char *name);
//! char      *rva_relative_for(const RvaHandle *handle, const char *name);
//! char      *rva_last_error(void);
//! void       rva_free_handle(RvaHandle *handle);
//! void       rva_free_string(char *ptr);
//! void       rva_free_buffer(uint8_t *ptr, size_t len);
//! ```
//!
//! Conventions:
//! - Returned `char *` values are owned by the caller and must be released with
//!   `rva_free_string` (except `rva_last_error`, which is thread-local and
//!   borrowed).
//! - Returned buffers are owned by the caller and must be released with
//!   `rva_free_buffer`.
//! - On failure, functions return NULL/0/false and set a message retrievable
//!   via `rva_last_error`.
//! - All functions are panic-safe (panics are caught and reported as errors).

#[cfg(feature = "jni")]
mod jni_bridge;

#[cfg(feature = "render")]
use rva_core::render_to_png;
use rva_core::{resolve, Asset, Fonts, ResolvedScene};
use std::cell::RefCell;
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

/// Opaque handle to a parsed asset and its font context.
pub struct RvaHandle {
    asset: Asset,
    fonts: Fonts,
}

thread_local! {
    static LAST_ERROR: RefCell<Option<CString>> = const { RefCell::new(None) };
}

fn set_error(message: impl ToString) {
    let text =
        CString::new(message.to_string()).unwrap_or_else(|_| CString::new("rva: error").unwrap());
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(text));
}

fn clear_error() {
    LAST_ERROR.with(|slot| *slot.borrow_mut() = None);
}

fn into_c_string(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(text) => text.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Borrow a `&str` from a C string pointer without taking ownership.
unsafe fn c_str<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err("null string pointer".to_string());
    }
    CStr::from_ptr(ptr)
        .to_str()
        .map_err(|_| "invalid UTF-8 string".to_string())
}

/// The current thread's last error message. Borrowed; do not free.
#[no_mangle]
pub extern "C" fn rva_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|message| message.as_ptr())
            .unwrap_or(ptr::null())
    })
}

/// Parse and integrity-validate a `.rva` package from memory.
///
/// # Safety
/// `data` must point to `len` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn rva_open(data: *const u8, len: usize) -> *mut RvaHandle {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<*mut RvaHandle, String> {
        if data.is_null() || len == 0 {
            return Err("empty input".to_string());
        }
        let bytes = std::slice::from_raw_parts(data, len).to_vec();
        let asset = Asset::from_bytes(bytes).map_err(|e| e.to_string())?;
        let handle = Box::new(RvaHandle {
            asset,
            fonts: Fonts::load_system(),
        });
        Ok(Box::into_raw(handle))
    }));

    match result {
        Ok(Ok(handle)) => {
            clear_error();
            handle
        }
        Ok(Err(message)) => {
            set_error(message);
            ptr::null_mut()
        }
        Err(_) => {
            set_error("panic while opening asset");
            ptr::null_mut()
        }
    }
}

/// Release a handle created by `rva_open`.
///
/// # Safety
/// `handle` must be a pointer returned by `rva_open`, or null.
#[no_mangle]
pub unsafe extern "C" fn rva_free_handle(handle: *mut RvaHandle) {
    if !handle.is_null() {
        drop(Box::from_raw(handle));
    }
}

/// Human-readable asset summary. Caller frees with `rva_free_string`.
///
/// # Safety
/// `handle` must be a valid pointer from `rva_open`.
#[no_mangle]
pub unsafe extern "C" fn rva_describe(handle: *const RvaHandle) -> *mut c_char {
    if handle.is_null() {
        set_error("null handle");
        return ptr::null_mut();
    }
    into_c_string((*handle).asset.describe())
}

/// Resolve the asset for a viewport, returning ResolvedScene JSON.
/// Caller frees with `rva_free_string`.
///
/// # Safety
/// `handle` must be a valid pointer from `rva_open`.
#[no_mangle]
pub unsafe extern "C" fn rva_resolve(
    handle: *const RvaHandle,
    width: u32,
    height: u32,
) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<String, String> {
        if handle.is_null() {
            return Err("null handle".to_string());
        }
        let handle = &*handle;
        let scene: ResolvedScene =
            resolve(&handle.asset, &handle.fonts, width, height).map_err(|e| e.to_string())?;
        serde_json::to_string(&scene).map_err(|e| e.to_string())
    }));

    match result {
        Ok(Ok(json)) => {
            clear_error();
            into_c_string(json)
        }
        Ok(Err(message)) => {
            set_error(message);
            ptr::null_mut()
        }
        Err(_) => {
            set_error("panic while resolving");
            ptr::null_mut()
        }
    }
}

/// Raw bytes for a declared resource id or raw path.
/// Sets `out_len` and returns a buffer the caller frees with `rva_free_buffer`.
///
/// # Safety
/// `handle` must be valid; `name` a NUL-terminated string; `out_len` non-null.
#[no_mangle]
pub unsafe extern "C" fn rva_resource(
    handle: *const RvaHandle,
    name: *const c_char,
    out_len: *mut usize,
) -> *mut u8 {
    if out_len.is_null() {
        set_error("null out_len");
        return ptr::null_mut();
    }
    *out_len = 0;

    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Vec<u8>, String> {
        if handle.is_null() {
            return Err("null handle".to_string());
        }
        let name = c_str(name)?;
        (*handle)
            .asset
            .reference_bytes(name)
            .map_err(|e| e.to_string())
    }));

    match result {
        Ok(Ok(bytes)) => {
            clear_error();
            let mut boxed = bytes.into_boxed_slice();
            let ptr = boxed.as_mut_ptr();
            *out_len = boxed.len();
            std::mem::forget(boxed);
            ptr
        }
        Ok(Err(message)) => {
            set_error(message);
            ptr::null_mut()
        }
        Err(_) => {
            set_error("panic while reading resource");
            ptr::null_mut()
        }
    }
}

/// Resolve and rasterize the asset to PNG bytes for a viewport.
/// Sets `out_len` and returns a buffer the caller frees with `rva_free_buffer`.
///
/// # Safety
/// `handle` must be valid; `out_len` non-null.
#[cfg(feature = "render")]
#[no_mangle]
pub unsafe extern "C" fn rva_render_png(
    handle: *const RvaHandle,
    width: u32,
    height: u32,
    out_len: *mut usize,
) -> *mut u8 {
    if out_len.is_null() {
        set_error("null out_len");
        return ptr::null_mut();
    }
    *out_len = 0;

    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Vec<u8>, String> {
        if handle.is_null() {
            return Err("null handle".to_string());
        }
        let handle = &*handle;
        let scene: ResolvedScene =
            resolve(&handle.asset, &handle.fonts, width, height).map_err(|e| e.to_string())?;
        render_to_png(&handle.asset, &scene, &handle.fonts).map_err(|e| e.to_string())
    }));

    match result {
        Ok(Ok(bytes)) => {
            clear_error();
            let mut boxed = bytes.into_boxed_slice();
            let ptr = boxed.as_mut_ptr();
            *out_len = boxed.len();
            std::mem::forget(boxed);
            ptr
        }
        Ok(Err(message)) => {
            set_error(message);
            ptr::null_mut()
        }
        Err(_) => {
            set_error("panic while rendering");
            ptr::null_mut()
        }
    }
}

/// Whether a resource reference exists in the package.
///
/// # Safety
/// `handle` must be valid; `name` a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rva_has_resource(handle: *const RvaHandle, name: *const c_char) -> bool {
    if handle.is_null() {
        return false;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        let name = c_str(name)?;
        Ok::<bool, String>((*handle).asset.has_reference(name))
    })) {
        Ok(Ok(found)) => found,
        _ => false,
    }
}

/// The path a reference resolves to, for MIME detection.
/// Caller frees with `rva_free_string`.
///
/// # Safety
/// `handle` must be valid; `name` a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rva_relative_for(
    handle: *const RvaHandle,
    name: *const c_char,
) -> *mut c_char {
    if handle.is_null() {
        set_error("null handle");
        return ptr::null_mut();
    }
    match c_str(name) {
        Ok(name) => into_c_string((*handle).asset.relative_for(name)),
        Err(message) => {
            set_error(message);
            ptr::null_mut()
        }
    }
}

/// Free a string returned by this library.
///
/// # Safety
/// `ptr` must come from this library, or be null.
#[no_mangle]
pub unsafe extern "C" fn rva_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Free a buffer returned by `rva_resource`.
///
/// # Safety
/// `ptr`/`len` must come from `rva_resource`, or `ptr` be null.
#[no_mangle]
pub unsafe extern "C" fn rva_free_buffer(ptr: *mut u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}
