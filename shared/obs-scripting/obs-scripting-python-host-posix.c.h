#include "shared/obs-scripting/build/obs-scripting-python-host-posix.pch.h"

bool obs_python_host_load_at_start(void)
{
	return false;
}

void obs_python_host_home(struct dstr *out, const char *python_path, int major, int minor)
{
	(void)python_path;
	(void)major;
	(void)minor;
	dstr_free(out);
}

bool obs_python_host_no_bytecode(void)
{
	return false;
}

void obs_python_host_module_dirs(obs_python_path_add_t add)
{
	(void)add;
}
