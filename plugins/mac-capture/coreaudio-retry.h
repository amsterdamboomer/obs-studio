/*
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

#pragma once

/* Retry schedule for the CoreAudio reconnect thread (mac-audio.c.h,
 * obs-rust/obs-studio#1358). Plain C with no Apple headers, so a cmocka
 * test drives it on every host with simulated time. */

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* The longest wait between two reconnect attempts. */
#define COREAUDIO_RETRY_MAX_MS 30000UL

struct coreaudio_retry {
	unsigned long base_ms;
	unsigned long next_ms;
	bool cap_reported;
};

/* Start (or restart, on a device-list or default-device change) the
 * schedule at base_ms. */
static inline void coreaudio_retry_reset(struct coreaudio_retry *r, unsigned long base_ms)
{
	r->base_ms = base_ms;
	r->next_ms = base_ms;
	r->cap_reported = false;
}

/* The wait before the next attempt. *cap_first is set to true exactly once
 * per schedule: the first time the wait reaches COREAUDIO_RETRY_MAX_MS. */
static inline unsigned long coreaudio_retry_next(struct coreaudio_retry *r, bool *cap_first)
{
	/* STUB (RED): today's fixed interval. */
	*cap_first = false;
	return r->base_ms;
}

#ifdef __cplusplus
}
#endif
