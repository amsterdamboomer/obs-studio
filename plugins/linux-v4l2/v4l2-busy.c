/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include "v4l2-busy.h"

/* RED: stubs, see obs-rust/obs-studio#799 */

bool v4l2_busy_errno(int err)
{
	(void)err;
	return false;
}

size_t v4l2_find_holders(const char *proc_root, const char *dev_path, pid_t self_pid, struct v4l2_holder *out,
			 size_t max)
{
	(void)proc_root;
	(void)dev_path;
	(void)self_pid;
	(void)out;
	(void)max;
	return 0;
}

bool v4l2_holder_is_obs(const struct v4l2_holder *holder)
{
	(void)holder;
	return false;
}

void v4l2_busy_describe(struct dstr *out, const char *dev_path, const struct v4l2_holder *holder)
{
	(void)dev_path;
	(void)holder;
	dstr_copy(out, "");
}

enum v4l2_force_result v4l2_force_release(const char *proc_root, const char *dev_path, const struct v4l2_holder *holder,
					  uid_t self_uid, uint32_t timeout_ms)
{
	(void)proc_root;
	(void)dev_path;
	(void)holder;
	(void)self_uid;
	(void)timeout_ms;
	return V4L2_FORCE_ERROR;
}

uint32_t v4l2_busy_retry_delay_ms(uint32_t attempt)
{
	(void)attempt;
	return 0;
}
