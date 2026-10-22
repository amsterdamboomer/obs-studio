#!/usr/bin/env -S uv run --script
"""Derive the package closure for prebuilt Tier 2 test runners."""
RUNTIME_TOOLS = ()


def library_paths(text):
    raise NotImplementedError


def package_names(text):
    raise NotImplementedError


def pinned_package(name, version):
    raise NotImplementedError


def elf_files(root):
    raise NotImplementedError
