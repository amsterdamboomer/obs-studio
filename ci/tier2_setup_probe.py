#!/usr/bin/env -S uv run --script
"""Measure stock and immutable prepared Tier 2 environments on fresh runners."""


def command(*args):
    raise NotImplementedError


def verify_bundle(root):
    raise NotImplementedError


def image_id(value):
    raise NotImplementedError


def inventory(text):
    raise NotImplementedError


def package_specs(text):
    raise NotImplementedError
