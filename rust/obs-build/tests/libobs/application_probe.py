#!/usr/bin/env -S uv run --script
"""Run a staged frontend with an owned scene/profile on an isolated X display."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import select
import subprocess
import time
import uuid


def profile(runtime: Path) -> Path:
    config = runtime / "user-config/obs-studio-rust"
    config.mkdir(parents=True)
    (config / "global.ini").write_text("[General]\nLastVersion=4294967295\n")
    (config / "user.ini").write_text(
        "[General]\nFirstRun=true\nConfirmOnExit=false\n"
        "[Basic]\nProfile=NativeSmoke\nProfileDir=NativeSmoke\n"
        "SceneCollection=NativeSmoke\nSceneCollectionFile=NativeSmoke\n"
    )
    selected = config / "basic/profiles/NativeSmoke"
    selected.mkdir(parents=True)
    (selected / "basic.ini").write_text(
        "[General]\nName=NativeSmoke\n[Video]\nBaseCX=640\nBaseCY=360\n"
        "OutputCX=640\nOutputCY=360\nFPSType=0\nFPSCommon=30\n"
        "[Audio]\nSampleRate=48000\nChannelSetup=Stereo\n"
    )
    color = {"name": "Native white", "id": "color_source_v3", "uuid": str(uuid.uuid4()),
             "settings": {"width": 640, "height": 360, "color": 0xFFFFFFFF}, "volume": 1.0}
    scene = {"name": "Native scene", "id": "scene", "uuid": str(uuid.uuid4()),
             "settings": {"items": [{"name": color["name"], "source_uuid": color["uuid"], "visible": True}]}}
    scenes = config / "basic/scenes"
    scenes.mkdir(parents=True)
    (scenes / "NativeSmoke.json").write_text(json.dumps({
        "name": "NativeSmoke", "current_scene": "Native scene", "current_program_scene": "Native scene",
        "scene_order": [{"name": "Native scene"}], "sources": [color, scene],
    }))
    return config


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--inspector", type=Path, required=True)
    args = parser.parse_args()
    runtime = args.runtime.resolve()
    config = profile(runtime)
    env = dict(os.environ)
    env.update(XDG_CONFIG_HOME=str(runtime / "user-config"), XDG_DATA_HOME=str(runtime / "user-data"),
               XDG_CACHE_HOME=str(runtime / "user-cache"), QT_QPA_PLATFORM="xcb",
               QT_QPA_PLATFORMTHEME="", QT_STYLE_OVERRIDE="Fusion")
    # Desktop-session paths may select plugins from a different Qt installation.
    # Use the linked Qt library's own default plugin paths for this isolated test.
    env.pop("QT_PLUGIN_PATH", None)
    env.pop("QT_QPA_PLATFORM_PLUGIN_PATH", None)
    env["LD_LIBRARY_PATH"] = str(runtime / "lib64") + os.pathsep + env.get("LD_LIBRARY_PATH", "")
    env.pop("OBS_DATA_PATH", None)
    env.pop("WAYLAND_DISPLAY", None)
    app = subprocess.Popen([str(runtime / "bin/obs-studio-rust"), "--multi", "--only-bundled-plugins",
                            "--disable-shutdown-check", "--disable-updater", "--disable-missing-files-check",
                            "--unfiltered_log"], env=env, cwd=runtime, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    log = bytearray()
    drained = False
    try:
        deadline = time.monotonic() + 45
        ready = False
        while time.monotonic() < deadline:
            if app.poll() is not None:
                raise RuntimeError(f"frontend exited during startup: {app.returncode}")
            if select.select([app.stdout], [], [], 0.2)[0]:
                chunk = os.read(app.stdout.fileno(), 65536)
                log.extend(chunk)
                (runtime / "application.log").write_bytes(log)
            if b"==== Startup complete" in log:
                found = subprocess.run([str(args.inspector), "find", str(app.pid)], env=env, capture_output=True)
                if found.returncode == 0:
                    ready = True
                    break
        if not ready:
            raise RuntimeError("frontend did not finish startup and map a normal main window")
        maps = Path(f"/proc/{app.pid}/maps").read_text()
        (runtime / "application.maps").write_text(maps)
        for name in ["libobs.so.30", "libobs-frontend-api.so.30", "libobs-opengl.so.33"]:
            assert str(runtime / "lib64" / name) in maps, f"staged {name} was not loaded"
        for name in json.loads((runtime / "expected-modules.json").read_text()):
            assert str(runtime / "lib64/obs-modules/core" / (name + ".so")) in maps, f"module {name} was not loaded"
        assert b"Native white" in log, "saved color source was not restored"
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "x11grab", "-video_size", "1280x720",
                        "-i", env["DISPLAY"], "-frames:v", "1", str(runtime / "application.png")], env=env,
                       check=True, capture_output=True, timeout=15)
        subprocess.run([str(args.inspector), "close", str(app.pid)], env=env, check=True, capture_output=True)
        remainder, _ = app.communicate(timeout=30)
        log.extend(remainder)
        drained = True
        (runtime / "application.log").write_bytes(log)
        assert app.returncode == 0, f"frontend shutdown failed: {app.returncode}"
        assert b"==== Shutting down" in log, "frontend did not enter normal shutdown"
        saved = json.loads((config / "basic/scenes/NativeSmoke.json").read_text())
        assert any(source.get("name") == "Native white" for source in saved["sources"]), "saved scene was lost"
        if os.environ.get("OBS_BUILD_APP_RECORD"):
            recordings = list((runtime / "recordings").glob("*.mkv"))
            assert len(recordings) == 1, "frontend did not create the requested recording"
        print(f"OBS normal window, staged libraries/modules, saved scene, screenshot and clean shutdown: {runtime}")
    finally:
        if not drained:
            if app.poll() is None:
                app.terminate()
            try:
                remainder, _ = app.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                app.kill()
                remainder, _ = app.communicate(timeout=5)
            log.extend(remainder)
        (runtime / "application.log").write_bytes(log)


if __name__ == "__main__":
    main()
