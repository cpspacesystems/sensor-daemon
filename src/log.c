#include <stdbool.h>
#include <stdio.h>
#include <fcntl.h>
#include <string.h>
#include <unistd.h>
#include <stdarg.h>

#include "log.h"

#define LOG_COLOR_RED "\e[0;31m"
#define LOG_COLOR_YELLOW "\e[0;33m"
#define LOG_COLOR_BLUE "\e[0;34m"
#define LOG_COLOR_MAGENTA "\e[0;35m"
#define LOG_COLOR_CLEAR "\e[0m"

#define LOG_L_TEXT_DEBUG "DBG"
#define LOG_L_TEXT_INFO "INF"
#define LOG_L_TEXT_WARN "WAR"
#define LOG_L_TEXT_ERROR "ERR"

static FILE* log_file = NULL;
static log_config_t log_config = 0;

static void log_print_level(FILE* f, log_level_t level, bool color) {
	if (color) {
		switch (level) {
			case LOG_L_DEBUG:
				fprintf(f, LOG_COLOR_MAGENTA LOG_L_TEXT_DEBUG LOG_COLOR_CLEAR);
				break;

			case LOG_L_INFO:
				fprintf(f, LOG_COLOR_BLUE LOG_L_TEXT_INFO LOG_COLOR_CLEAR);
				break;

			case LOG_L_WARN:
				fprintf(f, LOG_COLOR_YELLOW LOG_L_TEXT_WARN LOG_COLOR_CLEAR);
				break;

			case LOG_L_ERROR:
				fprintf(f, LOG_COLOR_RED LOG_L_TEXT_ERROR LOG_COLOR_CLEAR);
				break;
		}
	} else {
		switch (level) {
			case LOG_L_DEBUG:
				fprintf(f, LOG_L_TEXT_DEBUG);
				break;

			case LOG_L_INFO:
				fprintf(f, LOG_L_TEXT_INFO);
				break;

			case LOG_L_WARN:
				fprintf(f, LOG_L_TEXT_WARN);
				break;

			case LOG_L_ERROR:
				fprintf(f, LOG_L_TEXT_ERROR);
				break;
		}
	}
}

void log_init(const char* path) {
	if (!path) {
		log_file = NULL;
		return;
	}

	log_file = fopen(path, "w+");

	if (!log_file) {
		LOG_ERROR("Could not open log file!");
	}
}

void log_configure(log_config_t config) {
	log_config = config;
}

void log_record(const char* tag, log_level_t level, const char* msg, ...) {
	va_list vargs;
	va_start(vargs, msg);

	if (log_config & level) {
		FILE* file = stdout;

		if (level == LOG_L_ERROR) {
			file = stderr;
		}
	
		log_print_level(file, level, log_config & LOG_CONF_COLOR);
		fprintf(file, " [%s] ", tag); 
		vfprintf(file, msg, vargs);
		fprintf(file, "\n");
	}

	if (log_file > 0) {
		log_print_level(log_file, level, false);
		fprintf(log_file, " [%s] ", tag); 
		vfprintf(log_file, msg, vargs);
		fprintf(log_file, "\n");
	}

	va_end(vargs);
}
