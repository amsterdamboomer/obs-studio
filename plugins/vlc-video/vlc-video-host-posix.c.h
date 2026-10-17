#include "plugins/vlc-video/build/vlc-video-posix.pch.h"

/* RED stub (#857): not implemented yet. */
void *vlc_host_open_libvlc(void)
{
	return NULL;
}

void vlc_host_unload(void) {}

void vlc_host_native_path(struct dstr *path, bool is_url)
{
	UNUSED_PARAMETER(is_url);
	dstr_free(path);
}
