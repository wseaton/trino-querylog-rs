use std::env;

use jni::objects::JString;
use jni::sys::jlong;
use jni::{objects::JClass, JNIEnv};

use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

use tokio::runtime::Runtime;

pub struct QueryLogListener {
    config: ListenerConfig,
    kafka_producer: Option<FutureProducer>,
    runtime: Runtime,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ListenerConfig {
    #[serde(default = "default_true")]
    track_event_created: bool,
    #[serde(default = "default_true")]
    track_event_completed: bool,
    kafka_brokers: Option<String>,
    kafka_topic: Option<String>,
}

impl Default for ListenerConfig {
    fn default() -> Self {
        ListenerConfig {
            track_event_created: true,
            track_event_completed: true,
            kafka_brokers: None,
            kafka_topic: None,
        }
    }
}

fn default_true() -> bool {
    true
}

/// This is the Rust side of the `Plugin` class that is loaded by the Trino Plugin manager.
impl QueryLogListener {
    pub fn new(config: ListenerConfig) -> Result<QueryLogListener, Box<dyn std::error::Error>> {
        tracing::info!("QueryLogListener created");
        tracing::debug!("QueryLogListener config: {:?}", config);
        let runtime = Runtime::new()?;

        let kafka_producer = if let (Some(brokers), Some(_)) =
            (config.kafka_brokers.as_ref(), config.kafka_topic.as_ref())
        {
            let producer: Option<FutureProducer> = rdkafka::ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .create()
                .map_err(|e| {
                    tracing::error!("Failed to create Kafka producer: {}", e);
                    e
                })
                .ok();
            producer
        } else {
            None
        };

        if kafka_producer.is_none() {
            tracing::info!("Kafka log forwarding disabled!")
        }

        Ok(QueryLogListener {
            config,
            kafka_producer,
            runtime,
        })
    }

    fn run<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.runtime.block_on(future)
    }

    pub async fn query_created(&self, event: String) {
        if self.config.track_event_created {
            let event_json = serde_json::from_str::<serde_json::Value>(&event)
                .expect("Failed to parse event JSON");

            let span = tracing::info_span!("Event", event = %event_json);
            let _enter = span.enter();

            tracing::info!("Query created event");

            if let Some(producer) = &self.kafka_producer {
                let topic = self.config.kafka_topic.as_ref().unwrap();
                let _ = producer
                    .send(
                        FutureRecord::to(topic).payload(&event).key(""),
                        Timeout::Never,
                    )
                    .await;
            }
        }
    }

    pub async fn query_completed(&self, event: String) {
        if self.config.track_event_completed {
            let event_json = serde_json::from_str::<serde_json::Value>(&event)
                .expect("Failed to parse event JSON");

            let span = tracing::info_span!("Event", event = %event_json);
            let _enter = span.enter();

            tracing::info!("Query completed event");

            if let Some(producer) = &self.kafka_producer {
                let topic = self.config.kafka_topic.as_ref().unwrap();
                let _ = producer
                    .send(
                        FutureRecord::to(topic).payload(&event).key(""),
                        Timeout::Never,
                    )
                    .await;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_com_github_trino_querylog_QueryLogPlugin_00024Companion_initializeLogging(
    _env: JNIEnv,
    _class: JClass,
) {
    // Get the log file path from the environment variable, default to '/var/log/trino/querylog-rs/.log'
    let log_file_path = env::var("LOG_FILE_DIR").unwrap_or("/var/log/trino/".to_string());
    // Check if the LOG_TO_FILE environment variable is set
    if env::var("LOG_TO_FILE").is_ok() {
        let file_appender =
            RollingFileAppender::new(Rotation::NEVER, log_file_path, "querylog-rs.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer(non_blocking)
            .with_span_events(FmtSpan::CLOSE)
            .init();
    } else {
        // Default to a JSON log format that goes to STDOUT via EnvFilter
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }

    tracing::info!("Logging subsystem initialized");
}

#[no_mangle]
pub extern "C" fn Java_com_github_trino_querylog_QueryLogPlugin_00024Companion_createRustEventListener(
    env: JNIEnv,
    _class: JClass,
    config_obj: JString,
) -> jlong {
    let config = match jobject_str_to_listenerconfig(env, config_obj) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to parse config: {}", e);
            ListenerConfig::default()
        }
    };

    tracing::info!("Creating rust event listener");
    match QueryLogListener::new(config) {
        Ok(listener) => {
            let event_listener = Box::new(listener);
            Box::into_raw(event_listener) as jlong
        }
        Err(e) => {
            tracing::error!("Failed to create QueryLogListener: {}", e);
            0 // This signifies an empty object back through JNI
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
    tracing::debug!("FFI start - query created");
    let event_listener = unsafe { &*(rust_event_listener_ptr as *mut QueryLogListener) };
    let event: String = env
        .get_string(&query_created_event)
        .expect("Couldn't get java string!")
        .into();

    event_listener.run(event_listener.query_created(event));
}

#[no_mangle]
pub extern "C" fn Java_com_github_trino_querylog_JavaEventListenerWrapper_rustQueryCompleted(
    mut env: JNIEnv,
    _this: JClass,
    rust_event_listener_ptr: jlong, // jlong in JNI is mapped to jlong in Rust
    query_completed_event: JString,
) {
    tracing::debug!("FFI start - query completed");
    let event_listener = unsafe { &*(rust_event_listener_ptr as *mut QueryLogListener) };
    let event: String = env
        .get_string(&query_completed_event)
        .expect("Couldn't get java string!")
        .into();

    event_listener.run(event_listener.query_completed(event));
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

fn jobject_str_to_listenerconfig(
    mut env: JNIEnv,
    map_obj: JString,
) -> Result<ListenerConfig, jni::errors::Error> {
    tracing::debug!("Start converting config");
    let map_str: String = env.get_string(&map_obj)?.into();
    tracing::debug!("JObject converted to string: {}", map_str);
    let map: ListenerConfig = serde_java_properties::from_str(&map_str).map_err(|e| {
        jni::errors::Error::ParseFailed(
            combine::error::StringStreamError::UnexpectedParse,
            format!("Failed to convert config string: {}", e),
        )
    })?;
    tracing::debug!("JObject successfully converted to ListenerConfig");
    Ok(map)
}
