use std::collections::HashMap;

use jni::objects::JString;
use jni::sys::{jlong, JNI_TRUE};
use jni::{objects::JClass, objects::JObject, JNIEnv};

pub struct QueryLogListener {
    track_event_created: bool,
    _config: HashMap<String, String>,
}

/// This is the Rust side of the `Plugin` class that is loaded by the Trino Plugin manager.
impl QueryLogListener {
    pub fn new(track_event_created: bool, config: HashMap<String, String>) -> QueryLogListener {
        tracing::info!("QueryLogListener created");
        config.iter().for_each(|(k, v)| {
            tracing::debug!("{}: {}", k, v);
        });
        QueryLogListener {
            track_event_created,
            _config: config,
        }
    }

    pub fn query_created(&self, event: String) {
        if self.track_event_created {
            tracing::info!("Query created from rust: {}", event);
        }
    }

    pub fn query_completed(&self, event: String) {
        tracing::info!("Query completed from rust: {}", event);
    }
}

#[no_mangle]
pub extern "C" fn Java_com_github_trino_querylog_QueryLogPlugin_00024Companion_initializeLogging(
    _env: JNIEnv,
    _class: JClass,
) {
    tracing_subscriber::fmt().init();
    tracing::info!("Logging initialized");
}

#[no_mangle]
pub extern "C" fn Java_com_github_trino_querylog_QueryLogPlugin_00024Companion_createRustEventListener(
    env: &mut JNIEnv,
    _class: JClass,
    track_event_created: bool,
    config_obj: JObject,
) -> jlong {
    match jobject_to_hashmap(env, config_obj) {
        Ok(config) => {
            tracing::info!("Creating rust event listener");
            let event_listener = Box::new(QueryLogListener::new(track_event_created, config));
            Box::into_raw(event_listener) as jlong
        }
        Err(e) => {
            tracing::warn!("Failed to convert JObject to HashMap: {:?}", e);
            // Return 0, which corresponds to a null pointer in Java to indicate failure
            tracing::info!("Creating rust event listener");
            let event_listener = Box::new(QueryLogListener::new(
                track_event_created,
                HashMap::default(),
            ));
            Box::into_raw(event_listener) as jlong
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_com_github_trino_querylog_JavaEventListenerWrapper_rustQueryCreated(
    mut env: JNIEnv,
    _this: JClass,
    rust_event_listener_ptr: jlong, // jlong in JNI is mapped to jlong in Rust
    query_created_event: JString,
) {
    tracing::info!("Rust query created");
    let event_listener = unsafe { &*(rust_event_listener_ptr as *mut QueryLogListener) };
    let event: String = env
        .get_string(&query_created_event)
        .expect("Couldn't get java string!")
        .into();
    event_listener.query_created(event);
}

#[no_mangle]
pub extern "C" fn Java_com_github_trino_querylog_JavaEventListenerWrapper_rustQueryCompleted(
    mut env: JNIEnv,
    _this: JClass,
    rust_event_listener_ptr: jlong, // jlong in JNI is mapped to jlong in Rust
    query_completed_event: JString,
) {
    tracing::info!("Rust query completed");
    let event_listener = unsafe { &*(rust_event_listener_ptr as *mut QueryLogListener) };
    let event: String = env
        .get_string(&query_completed_event)
        .expect("Couldn't get java string!")
        .into();
    event_listener.query_completed(event);
}

// This is to facilitate freeing the memory allocated for the rust event listener,
// basically allowing the Java GC to Drop
#[no_mangle]
pub extern "C" fn Java_com_github_trino_querylog_JavaEventListenerWrapper_freeRustEventListener(
    _env: JNIEnv,
    _class: JClass,
    rust_event_listener_ptr: jlong,
) {
    unsafe {
        let _ = Box::from_raw(rust_event_listener_ptr as *mut QueryLogListener);
    }
}

// FIXME: this doesn't currently work, and we need a way to generically convert
// a Java Map<String, String> to a Rust HashMap<String, String> for config.
fn jobject_to_hashmap(
    env: &mut JNIEnv,
    map_obj: JObject,
) -> Result<HashMap<String, String>, jni::errors::Error> {
    let entry_set = env
        .call_method(map_obj, "entrySet", "()Ljava/util/Set;", &[])?
        .l()?;
    let iter = env
        .call_method(entry_set, "iterator", "()Ljava/util/Iterator;", &[])?
        .l()?;

    let mut hash_map = HashMap::new();

    while env.call_method(&iter, "hasNext", "()Z", &[])?.z()? == (JNI_TRUE == 1) {
        let entry = env
            .call_method(&iter, "next", "()Ljava/lang/Object;", &[])?
            .l()?;
        let key_obj = env
            .call_method(&entry, "getKey", "()Ljava/lang/Object;", &[])?
            .l()?;
        let value_obj = env
            .call_method(entry, "getValue", "()Ljava/lang/Object;", &[])?
            .l()?;

        let key: JString = JString::from(key_obj);
        let value: JString = JString::from(value_obj);

        let key_rust = env.get_string(&key)?.into();
        let value_rust = env.get_string(&value)?.into();

        hash_map.insert(key_rust, value_rust);
    }

    Ok(hash_map)
}
