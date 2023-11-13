# trino-querylog-rs

Trino Plugin to support log forwarding and analysis of query events, written in Kotlin + Rust using JNI.

This project draws heavy inspiration from <https://github.com/rchukh/trino-querylog/tree/master> on the initial design.

Why?

To be able to get near real-time introspection into how queries are doing, including information on query plans to create things like alerts.

Relevant Trino Docs:

- <https://trino.io/docs/current/develop/event-listener.html>
- <https://trino.io/docs/current/develop/spi-overview.html>

## Deploying

```dockerfile
# Use the official Trino image as the base
FROM trinodb/trino:latest
# Set the environment variable to point to the plugin directory
ENV TRINO_PLUGIN_DIR=/usr/lib/trino/plugin
# Copy your plugin JAR to the plugin directory in the container
COPY build/libs/trino-querylog-rs-1.0-SNAPSHOT.jar $TRINO_PLUGIN_DIR/querylog-rs/
# Copy your Rust shared library to the plugin directory in the container
COPY ./libtrino_querylog_rs.so /usr/local/lib/libtrino_querylog_rs.so
```

Add the following to `/etc/event-listener.properties`:

```ini
event-listener.name=rust-querylog-event-listener
### example config
# whether or not to log query create events
track_event_created=true
# whether or not to log query completed events
track_event_completed=true
### kafka configuration is optional
kafka_brokers=localhost:9091
kafka_topic=trino-query-logs
```

The built-in logging system can be configured via `EnvFilter` eg. `RUST_LOG`, as well as the `LOG_FILE_DIR` and `LOG_TO_FILE` env vars. By default the log forwarder will log to stdout which is interlaced with the Trino service logs.

Example:

To log to a rotated logfile in the directory `/tmp/logs` at loglevel DEBUG:

```sh
export RUST_LOG=debug
export LOG_FILE_DIR=/tmp/logs
export LOG_TO_FILE=true
```

## Developing

Commands are in the justfile:

```sh
just build
```

Will build the project, compiling the fat plugin jar and the shared library, and building a test docker image.

```sh
just run
```

Will do all of that, and also spin up a trino container for local testing.

### TODO

- [x] Config passing
- [x] Log forwarding (Kafka maybe?)
- [x] Log configuration
  - [x] Seperate file? JSON? Levels?
- [ ] Rust native types
- [ ] Analysis or rules engine and dynamic dispatch of alerts
