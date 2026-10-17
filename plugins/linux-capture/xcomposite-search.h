/* When the XComposite window capture searches for its window again, and
 * whether a search result needs a new pixmap and watcher (#1357).
 *
 * Pure: time and window state come in as arguments, so the decision is
 * tested without an X server (test/cmocka/test_xcomposite_search_backoff.c). */

#pragma once

#include <stdbool.h>
#include <stdint.h>

#define XCOMP_SEARCH_INTERVAL_MIN 2.0
#define XCOMP_SEARCH_INTERVAL_MAX 30.0

struct xcomp_search {
	double since_search;
	double interval;
};

/* Forget the backoff: the next search waits the shortest interval. */
static inline void xcomp_search_reset(struct xcomp_search *st)
{
	st->since_search = 0.0;
	st->interval = XCOMP_SEARCH_INTERVAL_MIN;
}

/* Called every tick. True when the window search runs now. */
static inline bool xcomp_search_due(struct xcomp_search *st, double seconds, bool window_lost, bool forced)
{
	(void)st;
	(void)seconds;
	(void)window_lost;
	(void)forced;
	return true; /* RED stub */
}

/* After a search: true when the pixmap and the watcher must be rebuilt. */
static inline bool xcomp_search_needs_rebuild(uint32_t old_win, uint32_t found_win, bool found_viewable, bool forced)
{
	(void)old_win;
	(void)found_win;
	(void)found_viewable;
	(void)forced;
	return true; /* RED stub */
}

/* After a search: captured is whether a texture exists now. */
static inline void xcomp_search_done(struct xcomp_search *st, bool captured, bool forced)
{
	(void)captured;
	(void)forced;
	st->since_search = 0.0;
}
