# Use the official Trino image as the base
FROM trinodb/trino:latest

# Set the environment variable to point to the plugin directory
ENV TRINO_PLUGIN_DIR=/usr/lib/trino/plugin

COPY ./etc /etc/trino/

USER root

RUN mkdir -p /var/log/trino && chmod 0777 /var/log/trino

USER trino

# Copy your plugin JAR to the plugin directory in the container
COPY build/libs/trino-querylog-rs-1.0-SNAPSHOT.jar $TRINO_PLUGIN_DIR/querylog-rs/

# Copy your Rust shared library to the plugin directory in the container
COPY ./libtrino_querylog_rs.so /usr/local/lib/libtrino_querylog_rs.so

