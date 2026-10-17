#pragma once

/* Waiting for an ingest-list refresh (service-ingest and dacast).
 *
 * The refresh runs on the file-updater thread; the caller (the output's
 * connect thread via rtmp_common_url, or the UI thread via the
 * *_ingests_refresh proc handlers) blocks until the refresh attempt ends or
 * the timeout passes. */

#include <util/threading.h>
#include <util/platform.h>

struct ingest_refresh_wait {
	volatile bool succeeded;
};

static inline void ingest_refresh_wait_init(struct ingest_refresh_wait *w)
{
	w->succeeded = false;
}

static inline void ingest_refresh_wait_free(struct ingest_refresh_wait *w)
{
	(void)w;
}

/* Called before a new refresh starts. */
static inline void ingest_refresh_wait_reset(struct ingest_refresh_wait *w)
{
	os_atomic_set_bool(&w->succeeded, false);
}

/* Called by the refresh when its attempt ends, whatever the outcome. */
static inline void ingest_refresh_wait_complete(struct ingest_refresh_wait *w, bool success)
{
	if (success)
		os_atomic_set_bool(&w->succeeded, true);
}

/* Whether a caller has to wait: only when no ingest list is available yet
 * (none from the cache, none from an earlier refresh). */
static inline bool ingest_refresh_needs_wait(bool refreshed, bool loaded)
{
	(void)refreshed;
	return !loaded;
}

/* Blocks until the refresh attempt ends or timeout_ms passes. Returns true
 * when the attempt ended. */
static inline bool ingest_refresh_wait_for(struct ingest_refresh_wait *w, unsigned long timeout_ms)
{
	for (unsigned long i = 0; i < timeout_ms / 10; i++) {
		if (os_atomic_load_bool(&w->succeeded))
			return true;
		os_sleep_ms(10);
	}
	return os_atomic_load_bool(&w->succeeded);
}
