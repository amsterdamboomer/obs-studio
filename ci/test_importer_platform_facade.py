#!/usr/bin/env -S uv run --script
"""Importer host boundaries and legacy path/source behavior (#881)."""

import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import platform_ifdef_ratchet as ratchet  # noqa: E402
from test_settings_platform_facade import function  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
OPERATIONS = ("PlatformImporterLastSeparator", "GetPlatformClassicImportDirectory",
              "GetPlatformStreamlabsImportDirectory", "GetPlatformXSplitImportDirectory",
              "TranslatePlatformImportedSource")

# Expected mappings from the pre-facade shared implementation. Settings are
# preserved even for ClearTranslation: its temporary empty settings object is
# overwritten at the end. Characterized here, not endorsed or repaired (#881).
MAPPINGS = {
    "mac": dict(zip(
        ["text_gdiplus", "game_capture", "wasapi_input_capture", "wasapi_output_capture",
         "pulse_input_capture", "pulse_output_capture", "jack_output_capture", "alsa_input_capture",
         "dshow_input", "v4l2_input", "xcomposite_input", "xshm_input"],
        ["text_ft2_source", "syphon-input", "coreaudio_input_capture", "coreaudio_output_capture",
         "coreaudio_input_capture", "coreaudio_output_capture", "coreaudio_output_capture",
         "coreaudio_input_capture", "av_capture_input", "av_capture_input", "window_capture", "monitor_capture"])),
    "win": dict(zip(
        ["text_ft2_source", "syphon-input", "coreaudio_input_capture", "coreaudio_output_capture",
         "pulse_input_capture", "pulse_output_capture", "jack_output_capture", "alsa_input_capture",
         "av_capture_input", "v4l2_input", "xcomposite_input"],
        ["text_gdiplus", "game_capture", "wasapi_input_capture", "wasapi_output_capture",
         "wasapi_input_capture", "wasapi_output_capture", "wasapi_output_capture",
         "wasapi_input_capture", "dshow_input", "dshow_input", "window_capture"])),
    "nix": dict(zip(
        ["text_gdiplus", "coreaudio_input_capture", "coreaudio_output_capture",
         "wasapi_input_capture", "wasapi_output_capture", "av_capture_input", "dshow_input"],
        ["text_ft2_source", "pulse_input_capture", "pulse_output_capture", "pulse_input_capture",
         "pulse_output_capture", "v4l2_input", "v4l2_input"])),
}


class ImporterFacadeTest(unittest.TestCase):
    def test_shared_files_have_no_host_decisions(self):
        for name in ("importers/classic.cpp.hpp", "importers/sl.cpp.hpp", "importers/studio.cpp.hpp",
                     "importers/xsplit.cpp.hpp", "importers/importers.hpp", "build/frontend-importers.pch.hpp"):
            with self.subTest(file=name):
                self.assertEqual(ratchet.count_platform_conditionals((ROOT / "frontend" / name).read_text()), [])

    def test_each_platform_defines_the_operations(self):
        header = (ROOT / "frontend/utility/platform.hpp").read_text()
        for operation in OPERATIONS:
            self.assertIn(operation + "(", header)
            for host in ("x11.cpp", "windows.cpp", "osx.mm"):
                with self.subTest(operation=operation, host=host):
                    self.assertRegex((ROOT / f"frontend/utility/platform-{host}.hpp").read_text(),
                                     rf"\b{operation}\([^;]*?\)\s*\{{")

    def test_legacy_paths_and_translation_matrix(self):
        compiler = shutil.which("c++")
        if not compiler:
            self.skipTest("C++ compiler unavailable; native builds still required")
        header = (ROOT / "frontend/importers/importers.hpp").read_text()
        shared = (ROOT / "frontend/importers/studio.cpp.hpp").read_text()
        paths = ["", "file.txt", "dir/file.txt", r"dir\file.txt", r"a/b\file.txt",
                 r"a\b/file.txt", "/a.b/file", "/a/.hidden", "dir/", r"C:\a\b.ext"]
        ids = sorted(set().union(*(mapping.keys() for mapping in MAPPINGS.values())) |
                     {"unknown", "window_capture", "monitor_capture"})
        for host, kind, macro in (("windows.cpp", "win", "_WIN32"),
                                  ("osx.mm", "mac", "__APPLE__"),
                                  ("x11.cpp", "nix", "__linux__")):
            with self.subTest(host=host):
                text = (ROOT / f"frontend/utility/platform-{host}.hpp").read_text()
                # During RED, characterize the original shared code. GREEN
                # reads the new platform function and unchanged shared helpers.
                moved = "TranslatePlatformImportedSource(" in text
                bodies = [function(header, "static inline std::string " + name + "(")
                          for name in ("GetFilenameFromPath", "GetFolderFromPath")]
                if moved:
                    bodies.insert(0, function(text, "size_t PlatformImporterLastSeparator("))
                    bodies.append(function(text, "void TranslatePlatformImportedSource("))
                else:
                    bodies.append(function(shared, "void TranslateOSStudio("))
                    bodies.append('void TranslatePlatformImportedSource(Json &v) { Json root = Json::object{{"sources", Json::array{v}}}; TranslateOSStudio(root); v = root["sources"][0]; }')
                preamble = r'''
#include <string>
#include <iostream>
#include "json11.hpp"
using namespace std;
using namespace json11;
constexpr int OBS_NIX_PLATFORM_X11_EGL = 1;
int session = 1;
int obs_get_nix_platform() { return session; }
const char *obs_get_latest_input_type_id(const char *id) {
    static string out; out = string("latest:") + id; return out.c_str();
}
'''
                # Host macros apply only to the extracted legacy functions,
                # never to the standard library or json11 build.
                code = preamble + f"\n#define {macro} 1\n" + "\n".join(bodies) + r'''
int main() {
    string line;
    while (getline(cin, line)) {
        string error; Json input = Json::parse(line, error);
        session = input["session"].int_value();
        if (input["path"].is_string()) {
            string p = input["path"].string_value();
            cout << Json(Json::array{GetFilenameFromPath(p), GetFolderFromPath(p)}).dump() << '\n';
        } else {
            Json source = input["source"]; TranslatePlatformImportedSource(source);
            cout << source.dump() << '\n';
        }
    }
}
'''
                requests, expected = [], []
                for path in paths:
                    slash, back = path.rfind('/'), path.rfind('\\')
                    # Preserve the Windows npos comparison quirk: when '/' is
                    # absent it wins over a real backslash position.
                    pos = max(slash, back) if kind == "win" and slash != -1 else slash
                    ext = path.rfind('.')
                    filename = path[pos + 1:] if ext < pos or pos == -1 else path[pos + 1:ext]
                    requests.append({"path": path}); expected.append([filename, path[:pos + 1]])
                for session in (1, 2):
                    for id_ in ids:
                        for settings in ({}, {"keep": 7, "capture_cursor": False},
                                         {"keep": 7, "show_cursor": False, "capture_cursor": True}):
                            source = {"id": id_, "versioned_id": "original", "settings": settings,
                                      "other": "preserved"}
                            out = json.loads(json.dumps(source)); mapping = dict(MAPPINGS[kind])
                            if kind == "nix":
                                mapping.update({x: "xcomposite_input" if session == 1 else
                                                "pipewire-screen-capture-source" for x in ("game_capture", "window_capture")})
                            if id_ in mapping:
                                out.update(id=mapping[id_], versioned_id="latest:" + mapping[id_])
                            if kind == "win" and id_ in ("monitor_capture", "xshm_input"):
                                out["id"] = "monitor_capture"
                                out["settings"].setdefault("show_cursor", None)
                                if settings.get("show_cursor") is not None:
                                    out["settings"]["capture_cursor"] = settings["show_cursor"]
                            elif kind != "win" and id_ == "monitor_capture":
                                if kind == "nix":
                                    out["id"] = "xshm_input"
                                out["settings"].setdefault("show_cursor", None)
                                if settings.get("show_cursor") is None:
                                    out["settings"].setdefault("capture_cursor", None)
                                if settings.get("show_cursor") is None and settings.get("capture_cursor") is not None:
                                    out["settings"]["show_cursor"] = settings["capture_cursor"]
                            requests.append({"session": session, "source": source}); expected.append(out)
                with tempfile.TemporaryDirectory(prefix="obs-importer-policy-") as tmp:
                    source, binary = Path(tmp) / "probe.cpp", Path(tmp) / "probe"
                    source.write_text(code)
                    subprocess.run([compiler, "-std=c++17", "-I", str(ROOT / "deps/json11"),
                                    str(source), str(ROOT / "deps/json11/json11.cpp"), "-o", str(binary)],
                                   check=True, capture_output=True)
                    result = subprocess.run([str(binary)], input="\n".join(map(json.dumps, requests)) + "\n",
                                            text=True, capture_output=True, check=True)
                    actual = [json.loads(line) for line in result.stdout.splitlines()]
                    self.assertEqual(len(actual), len(expected))
                    for request, want, got in zip(requests, expected, actual):
                        with self.subTest(host=host, request=request):
                            self.assertEqual(got, want)


if __name__ == "__main__":
    unittest.main()
