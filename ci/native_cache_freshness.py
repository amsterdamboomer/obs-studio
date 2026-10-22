#!/usr/bin/env -S uv run --script
"""Temporary source overlay and installed-binary evidence for native cache qualification."""


def prepare(root, state, context):
    raise NotImplementedError


def verify(root, state, platform, architecture):
    raise NotImplementedError


def restore(root, state):
    raise NotImplementedError
