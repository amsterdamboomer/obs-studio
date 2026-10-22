#!/usr/bin/env -S uv run --script
"""Capture native runner toolchain identity before compiler-cache restoration."""


def capture(platform, architecture, configuration):
    raise NotImplementedError
