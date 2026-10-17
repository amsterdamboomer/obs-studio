#include "plugins/obs-qsv11/build/obs-qsv11-windows-c.pch.h"

bool qsv_host_texture_valid(const struct encoder_texture *tex)
{
	(void)tex;
	return false; /* RED stub (#860) */
}
