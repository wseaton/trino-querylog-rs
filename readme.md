# trino-querylog-rs

PoC Trino Plugin to support log forwarding and analysis of query events, written in Kotlin + Rust using JNI.

This project draws heavy inspiration from <https://github.com/rchukh/trino-querylog/tree/master> on the initial design.

Why?

To be able to get near real-time introspection into how queries are doing, including information on query plans to create things like alerts.

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

- [ ] Config passing
- [ ] Log forwarding (Kafka maybe?)
- [ ] Rust native types
- [ ] Analysis or rules engine and dynamic dispatch of alerts
