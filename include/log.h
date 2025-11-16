/*
 * File based, as well as stdout and stderr based logging.
 */

#ifndef _LOG_H
#define _LOG_H

#include <stdint.h>

/** Log a `printf` style message at `LOG_L_DEBUG`. */
#define LOG_DEBUG(...) log_record(__FILE_NAME__, LOG_L_DEBUG, __VA_ARGS__)
/** Log a `printf` style message at `LOG_L_INFO`. */
#define LOG_INFO(...) log_record(__FILE_NAME__, LOG_L_INFO, __VA_ARGS__)
/** Log a `printf` style message at `LOG_L_WARN`. */
#define LOG_WARN(...) log_record(__FILE_NAME__, LOG_L_WARN, __VA_ARGS__)
/** Log a `printf` style message at `LOG_L_ERROR`. */
#define LOG_ERROR(...) log_record(__FILE_NAME__, LOG_L_ERROR, __VA_ARGS__)

/**
 * Log levels, these are successively higher "urgencies" with which a message
 * can be logged. This interface makes not real evaluation of higher or lower
 * urgency messages, and each level can be toggle individually.
 *
 * Further, as values these are all successive bits, not values, so they can be
 * used a bit flags.
 */
typedef enum {
	LOG_L_DEBUG = 0x01,
	LOG_L_INFO = 0x02,
	LOG_L_WARN = 0x04,
	LOG_L_ERROR = 0x08,
} log_level_t;

typedef uint8_t log_config_t;

/** Enable `LOG_L_DEBUG` messages. */
#define LOG_CONF_L_ENABLE_DEBUG LOG_L_DEBUG
/** Enable `LOG_L_INFO` messages. */
#define LOG_CONF_L_ENABLE_INFO LOG_L_INFO
/** Enable `LOG_L_WARN` messages. */
#define LOG_CONF_L_ENABLE_WARN LOG_L_WARN
/** Enable `LOG_L_ERROR` messages. */
#define LOG_CONF_L_ENABLE_ERROR LOG_L_ERROR

/** Enable messages as verbose as `LOG_L_DEBUG`. */
#define LOG_CONF_DEBUG (LOG_CONF_INFO | LOG_CONF_L_ENABLE_DEBUG)
/** Enable messages as verbose as `LOG_L_INFO`. */
#define LOG_CONF_INFO (LOG_CONF_WARN | LOG_CONF_L_ENABLE_INFO)
/** Enable messages as verbose as `LOG_L_WARN`. */
#define LOG_CONF_WARN (LOG_CONF_ERROR | LOG_CONF_L_ENABLE_WARN)
/** Enable messages as verbose as `LOG_L_ERROR`. */
#define LOG_CONF_ERROR LOG_CONF_L_ENABLE_ERROR
/** Enable any and all log messages. */
#define LOG_CONF_ALL LOG_CONF_DEBUG

/** Enable color in console logs, color is always disabled in files. */
#define LOG_CONF_COLOR 0x10

/**
 * Initialize the log, opening the given file-path for logging. You may pass
 * `NULL` to ommit file based logging.
 */
void log_init(const char* path);

/**
 * Configure the logging. This only affects console logs, file based logging is
 * still going to log everything/
 */
void log_configure(log_config_t config);

/**
 * Record the given `msg`, at the given `log_level_t`, tagged with `tag`. You
 * need not include a newline, one will be appended by `log_record`.
 *
 * This function is variadic and works the same as the `printf` family of
 * functions.
 */
void log_record(const char* tag, log_level_t level, const char* msg, ...);

#endif  /* _LOG_H */
