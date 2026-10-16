/* Precompiled header: C standard library (cmake/common/pch.cmake).
 *
 * Force-included before every C source of a target that has no libobs
 * include path. Keep it to headers whose meaning doesn't depend on a macro a
 * source defines first; sources that define one before their first #include
 * set SKIP_PRECOMPILE_HEADERS instead (see pch.cmake). */

#pragma once

/* glibc extensions (pthread_setname_np, ...) that several sources request
 * with _GNU_SOURCE after other includes; C++ compilers define it already. */
#if defined(__linux__) && !defined(_GNU_SOURCE)
#define _GNU_SOURCE
#endif

/* M_PI and friends for MSVC's <math.h>; harmless elsewhere. */
#ifndef _USE_MATH_DEFINES
#define _USE_MATH_DEFINES
#endif

#include <assert.h>
#include <ctype.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <math.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <wchar.h>
