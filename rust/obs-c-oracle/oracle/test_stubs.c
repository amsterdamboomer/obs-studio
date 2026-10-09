/* Test-only stand-ins for libobs symbols (base.c, platform.c) that the
 * oracles and the Rust bmem shim reference under cargo test. Delete together
 * with the oracles once the C sources are deleted.
 *
 * Deliberately includes no libobs header: they mark these functions
 * EXPORT/dllexport. Prototypes below match libobs/util/base.h and
 * libobs/util/platform.h. */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wchar.h>

void *bmalloc(size_t size);
void bfree(void *ptr);

void bcrash(const char *format, ...);
void os_breakpoint(void);
void os_oom(void);
FILE *os_fopen(const char *path, const char *mode);
size_t os_fread_utf8(FILE *file, char **pstr);
size_t os_mbs_to_utf8_ptr(const char *str, size_t len, char **pstr);
size_t os_utf8_to_wcs_ptr(const char *str, size_t len, wchar_t **pstr);

void bcrash(const char *format, ...)
{
	va_list args;
	va_start(args, format);
	vfprintf(stderr, format, args);
	va_end(args);
	abort();
}

void os_breakpoint(void) {}

void os_oom(void) {}

FILE *os_fopen(const char *path, const char *mode)
{
	return path ? fopen(path, mode) : NULL;
}

/* Mirrors platform.c os_fread_utf8, including returning 0 for the length. */
size_t os_fread_utf8(FILE *file, char **pstr)
{
	size_t size = 0;
	size_t len = 0;

	*pstr = NULL;

	fseek(file, 0, SEEK_END);
	size = (size_t)ftell(file);

	if (size > 0) {
		char bom[3] = {0, 0, 0};
		char *utf8str;
		long offset;

		fseek(file, 0, SEEK_SET);
		size_t size_read = fread(bom, 1, 3, file);
		(void)size_read;

		offset = (memcmp(bom, "\xEF\xBB\xBF", 3) == 0) ? 3 : 0;

		size -= (size_t)offset;
		if (size == 0)
			return 0;

		utf8str = bmalloc(size + 1);
		fseek(file, offset, SEEK_SET);

		size = fread(utf8str, 1, size, file);
		if (size == 0) {
			bfree(utf8str);
			return 0;
		}

		utf8str[size] = 0;

		*pstr = utf8str;
	}

	return len;
}

/* Not exercised by the oracles or tests. */
size_t os_mbs_to_utf8_ptr(const char *str, size_t len, char **pstr)
{
	(void)str;
	(void)len;
	(void)pstr;
	abort();
}

size_t os_utf8_to_wcs_ptr(const char *str, size_t len, wchar_t **pstr)
{
	(void)str;
	(void)len;
	(void)pstr;
	abort();
}
