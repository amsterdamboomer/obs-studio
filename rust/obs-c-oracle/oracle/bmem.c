/* Test-only oracle: the original libobs/util/bmem.c, unmodified, with its
 * global symbols renamed to oracle_* so it can link next to the Rust
 * implementation. os_breakpoint/os_oom/bcrash resolve to oracle/test_stubs.c. */
#define bmalloc oracle_bmalloc
#define brealloc oracle_brealloc
#define bfree oracle_bfree
#define bnum_allocs oracle_bnum_allocs
#define base_get_alignment oracle_base_get_alignment
#define bmemdup oracle_bmemdup

#include "util/bmem.c"
