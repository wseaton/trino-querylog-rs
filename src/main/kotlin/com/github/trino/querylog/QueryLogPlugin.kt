package com.github.trino.querylog

import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.fasterxml.jackson.databind.SerializationFeature
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule

import io.trino.spi.eventlistener.EventListenerFactory;
import io.trino.spi.eventlistener.EventListener
import io.trino.spi.eventlistener.QueryCreatedEvent
import io.trino.spi.eventlistener.QueryCompletedEvent
import io.trino.spi.eventlistener.SplitCompletedEvent
import io.trino.spi.Plugin
import java.util.Collections

import org.slf4j.LoggerFactory

// Assuming ObjectMapper is a singleton or appropriately scoped instance
val objectMapper = ObjectMapper().apply {
    // Register the JavaTimeModule to handle Java 8 Date & Time API types
    registerModule(JavaTimeModule())
    // To serialize dates as Strings instead of timestamps (numeric)
    disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS)
}

class QueryLogPlugin : Plugin {
    companion object {
        private val logger = LoggerFactory.getLogger(QueryLogPlugin::class.java)

        init {
            logger.info("QueryLogPlugin is being initialized")
            try {
                System.load("/usr/local/lib/libtrino_querylog_rs.so")
                initializeLogging()
                logger.info("Native library loaded and logging initialized successfully.")
            } catch (e: Throwable) {
                logger.error("Error loading native library or initializing logging.", e)
            }
        }

        // Declare the external function that will create a Rust event listener
        external fun createRustEventListener(config: String): Long
        private external fun initializeLogging()
    }

    override fun getEventListenerFactories(): Iterable<EventListenerFactory> {
        logger.debug("Getting EventListenerFactories")
        return listOf(object : EventListenerFactory {
            override fun create(config: Map<String, String>): EventListener {
                logger.debug("Attempting to create a Rust EventListener")
                try {
                    val configAsString = objectMapper.writeValueAsString(config)
                    logger.debug("Config: $configAsString")
                    val eventListenerPtr = createRustEventListener(configAsString)
                    logger.debug("Rust EventListener created successfully")
                    return JavaEventListenerWrapper(eventListenerPtr)
                } catch (e: Throwable) {
                    logger.error("Failed to create Rust EventListener", e)
                    throw RuntimeException("Failed to create Rust EventListener", e)
                }
            }

            override fun getName(): String {
                return "rust-querylog-event-listener"
            }
        })
    }
}



class JavaEventListenerWrapper(private val rustEventListenerPtr: Long) : EventListener {

    private external fun rustQueryCreated(rustEventListenerPtr: Long, queryCreatedEvent: String)
    private external fun rustQueryCompleted(rustEventListenerPtr: Long, queryCompletedEvent: String)
    private external fun freeRustEventListener(rustEventListenerPtr: Long)

    override fun queryCreated(queryCreatedEvent: QueryCreatedEvent) {
        // Convert queryCreatedEvent to JSON or another string representation
        val eventAsString = convertEventToString(queryCreatedEvent)
        rustQueryCreated(rustEventListenerPtr, eventAsString)
    }

    override fun queryCompleted(queryCompletedEvent: QueryCompletedEvent) {
        // Convert queryCompletedEvent to JSON or another string representation
        val eventAsString = convertEventToString(queryCompletedEvent)
        rustQueryCompleted(rustEventListenerPtr, eventAsString)
    }

    // Helper method to convert QueryCreatedEvent to a string format
    private fun convertEventToString(queryCreatedEvent: QueryCreatedEvent): String {
        return objectMapper.writeValueAsString(queryCreatedEvent)
    }

    // Helper method to convert QueryCompletedEvent to a string format
    private fun convertEventToString(queryCompletedEvent: QueryCompletedEvent): String {
        return objectMapper.writeValueAsString(queryCompletedEvent)
    }



    @Throws(Throwable::class)
    protected fun finalize() {
        freeRustEventListener(rustEventListenerPtr)
    }
}