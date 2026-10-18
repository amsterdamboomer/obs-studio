#!/usr/bin/env -S uv run --script
"""Start a job-owned native cache journal and summarize it without raw arguments."""

def summarize(text, workspace, max_records=10000):
    raise NotImplementedError


def start(tool, workspace, cache_dir, journal, environ_file):
    raise NotImplementedError
