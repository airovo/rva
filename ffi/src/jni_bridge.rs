//! JNI bridge for the Android/Kotlin adapter.
//!
//! Exposes the same operations as the C ABI to a Kotlin `object RvaNative`
//! (dev.rva.RvaNative). Compiled into `librva_ffi.so` with `--features jni`.
//!
//! Using JNI directly (instead of JNA) keeps the app free of third-party native
//! libraries and lets the shared object be 16 KB page aligned for Android 15+.

use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jboolean, jbyteArray, jint, jlong, jstring, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;
use std::ptr;

use crate::{render_to_png, resolve, Asset, Fonts, RvaHandle};

fn throw(env: &mut JNIEnv, message: impl std::fmt::Display) {
    let _ = env.throw_new("java/lang/IllegalStateException", message.to_string());
}

unsafe fn handle_ref<'a>(handle: jlong) -> Option<&'a RvaHandle> {
    if handle == 0 {
        None
    } else {
        Some(&*(handle as *const RvaHandle))
    }
}

fn string_or_throw(env: &mut JNIEnv, value: String) -> jstring {
    match env.new_string(value) {
        Ok(string) => string.into_raw(),
        Err(error) => {
            throw(env, error);
            ptr::null_mut()
        }
    }
}

fn read_name(env: &mut JNIEnv, value: &JString) -> Result<String, String> {
    env.get_string(value)
        .map(|s| s.to_string_lossy().into_owned())
        .map_err(|error| error.to_string())
}

#[no_mangle]
pub extern "system" fn Java_dev_rva_RvaNative_nativeOpen<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data: JByteArray<'local>,
) -> jlong {
    let bytes = match env.convert_byte_array(&data) {
        Ok(bytes) => bytes,
        Err(error) => {
            throw(&mut env, error);
            return 0;
        }
    };
    match Asset::from_bytes(bytes) {
        Ok(asset) => Box::into_raw(Box::new(RvaHandle {
            asset,
            fonts: Fonts::load_system(),
        })) as jlong,
        Err(error) => {
            throw(&mut env, error);
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_rva_RvaNative_nativeFree<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    if handle != 0 {
        unsafe {
            drop(Box::from_raw(handle as *mut RvaHandle));
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_rva_RvaNative_nativeDescribe<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jstring {
    let text = unsafe { handle_ref(handle) }
        .map(|handle| handle.asset.describe())
        .unwrap_or_default();
    string_or_throw(&mut env, text)
}

#[no_mangle]
pub extern "system" fn Java_dev_rva_RvaNative_nativeResolve<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    width: jint,
    height: jint,
) -> jstring {
    let Some(handle) = (unsafe { handle_ref(handle) }) else {
        throw(&mut env, "null handle");
        return ptr::null_mut();
    };
    match resolve(&handle.asset, &handle.fonts, width as u32, height as u32)
        .map_err(|error| error.to_string())
        .and_then(|scene| serde_json::to_string(&scene).map_err(|error| error.to_string()))
    {
        Ok(json) => string_or_throw(&mut env, json),
        Err(error) => {
            throw(&mut env, error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_rva_RvaNative_nativeRenderPng<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    width: jint,
    height: jint,
) -> jbyteArray {
    let Some(handle) = (unsafe { handle_ref(handle) }) else {
        throw(&mut env, "null handle");
        return ptr::null_mut();
    };
    let rendered = resolve(&handle.asset, &handle.fonts, width as u32, height as u32)
        .map_err(|error| error.to_string())
        .and_then(|scene| {
            render_to_png(&handle.asset, &scene, &handle.fonts).map_err(|error| error.to_string())
        });
    match rendered {
        Ok(bytes) => match env.byte_array_from_slice(&bytes) {
            Ok(array) => array.into_raw(),
            Err(error) => {
                throw(&mut env, error);
                ptr::null_mut()
            }
        },
        Err(error) => {
            throw(&mut env, error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_rva_RvaNative_nativeResource<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    name: JString<'local>,
) -> jbyteArray {
    let Some(handle) = (unsafe { handle_ref(handle) }) else {
        throw(&mut env, "null handle");
        return ptr::null_mut();
    };
    let name = match read_name(&mut env, &name) {
        Ok(name) => name,
        Err(error) => {
            throw(&mut env, error);
            return ptr::null_mut();
        }
    };
    match handle.asset.reference_bytes(&name) {
        Ok(bytes) => match env.byte_array_from_slice(&bytes) {
            Ok(array) => array.into_raw(),
            Err(error) => {
                throw(&mut env, error);
                ptr::null_mut()
            }
        },
        Err(error) => {
            throw(&mut env, error);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_rva_RvaNative_nativeHasResource<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    name: JString<'local>,
) -> jboolean {
    let Some(handle) = (unsafe { handle_ref(handle) }) else {
        return JNI_FALSE;
    };
    match read_name(&mut env, &name) {
        Ok(name) if handle.asset.has_reference(&name) => JNI_TRUE,
        _ => JNI_FALSE,
    }
}

#[no_mangle]
pub extern "system" fn Java_dev_rva_RvaNative_nativeRelativeFor<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    name: JString<'local>,
) -> jstring {
    let Some(handle) = (unsafe { handle_ref(handle) }) else {
        return ptr::null_mut();
    };
    let name = match read_name(&mut env, &name) {
        Ok(name) => name,
        Err(error) => {
            throw(&mut env, error);
            return ptr::null_mut();
        }
    };
    string_or_throw(&mut env, handle.asset.relative_for(&name))
}
