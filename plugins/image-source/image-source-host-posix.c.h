#include "plugins/image-source/build/image-source-posix.pch.h"

/* Stubs (RED, #858). */
bool image_source_host_jxr_supported(void)
{
	return true;
}

const char *image_source_host_image_filter(void)
{
	return "";
}

const char *image_source_host_slideshow_filter(void)
{
	return "";
}

int64_t image_source_host_stat_mtime_nsec(const struct stat *stats)
{
	(void)stats;
	return -1;
}
