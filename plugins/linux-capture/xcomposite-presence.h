/* Whether the XComposite capture's window is viewable, kept from the X
 * events the watcher receives instead of a round trip per frame (#1356).
 *
 * Pure: time and events come in as arguments, so the decision is tested
 * without an X server (test/cmocka/test_xcomposite_presence.c). */

#pragma once

#include <stdbool.h>

/* The safety-net round trip runs at most this often (seconds) while the
 * cached state is known, in case an event was missed. */
#define XCOMP_PRESENCE_RECHECK 1.0

enum xcomp_presence_event {
	XCOMP_PRESENCE_MAPPED,
	XCOMP_PRESENCE_UNMAPPED,
	XCOMP_PRESENCE_DESTROYED,
	XCOMP_PRESENCE_OTHER,
};

struct xcomp_presence {
	bool known;
	bool viewable;
	double since_check;
};

/* The window changed (new window, new settings, window_changed): the next
 * tick asks the server. */
static inline void xcomp_presence_invalidate(struct xcomp_presence *st)
{
	(void)st;
}

/* An X event on the captured window. */
static inline void xcomp_presence_event(struct xcomp_presence *st, enum xcomp_presence_event ev)
{
	(void)st;
	(void)ev;
}

/* Called every tick. True when the tick must ask the server now. */
static inline bool xcomp_presence_check_due(struct xcomp_presence *st, double seconds)
{
	(void)st;
	(void)seconds;
	return true;
}

/* The answer of a round trip (xcomp_window_exists). */
static inline void xcomp_presence_set(struct xcomp_presence *st, bool viewable)
{
	st->known = true;
	st->viewable = viewable;
	st->since_check = 0.0;
}
