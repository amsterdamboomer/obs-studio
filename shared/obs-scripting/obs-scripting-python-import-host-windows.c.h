#include "shared/obs-scripting/build/obs-scripting-python-import-host-windows.pch.h"

void obs_python_host_lib_name(struct dstr *out, int major, int minor)
{
	(void)major;
	(void)minor;
	dstr_free(out);
}
