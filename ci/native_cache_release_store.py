#!/usr/bin/env -S uv run --script
"""GitHub release asset adapter for the bounded native-cache protocol."""


class ReleaseStore:
    def __init__(self, identity, token='', writable=False):
        raise NotImplementedError
