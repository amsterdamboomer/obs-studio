#pragma once

/* Private (#874): opens profiler-io.c.h's csv.gz for writing; the host files
 * profiler-io-host-<os>.c.h implement it. NULL on failure. */

struct gzFile_s;

struct gzFile_s *profiler_gzopen(const char *filename);
