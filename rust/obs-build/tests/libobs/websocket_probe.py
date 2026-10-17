#!/usr/bin/env -S uv run --script
# /// script
# dependencies = ["websockets==13.1"]
# ///
"""Exercise the staged plugin using the unchanged Tier 4 wire client."""
from __future__ import annotations

import argparse
import asyncio
import base64
import json
import os
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[4] / "test/websocket-tier4"))
from obsws import ObsWsClient, SUB_SCENES  # noqa: E402


async def verify(runtime: Path, recording: bool) -> None:
    manifest = json.loads((runtime / "expected-websocket.json").read_text())
    uri = f"ws://127.0.0.1:{manifest['port']}"
    bad = ObsWsClient(uri)
    try:
        hello = await bad.connect()
        assert hello and hello["d"]["rpcVersion"] == 1
        challenge = hello["d"]["authentication"]
        await bad.identify(password="incorrect-fixture-password", auth_challenge=challenge)
        await asyncio.wait_for(bad.ws.wait_closed(), 5)
        assert bad.ws.close_code == 4009, "server accepted incorrect password"
    finally:
        await bad.close_conn()

    client = ObsWsClient(uri)
    evidence = {}
    try:
        hello = await client.connect()
        assert hello and "authentication" in hello["d"]
        await client.identify(password=manifest["password"],
                              auth_challenge=hello["d"]["authentication"],
                              event_subscriptions=SUB_SCENES)
        assert await client.wait_op(2), "authenticated Identify failed"

        async def request(name: str, data=None):
            reply = await client.request(name, data)
            assert reply["d"]["requestStatus"]["code"] == 100, reply
            result = reply["d"].get("responseData", {})
            evidence[name] = result
            return result

        version = await request("GetVersion")
        assert version["obsWebSocketVersion"] == "5.7.5"
        scenes = await request("GetSceneList")
        assert scenes["currentProgramSceneName"] == "Native scene"
        inputs = await request("GetInputList")
        assert any(item["inputName"] == "Native white" for item in inputs["inputs"])
        settings = await request("GetInputSettings", {"inputName": "Native white"})
        assert settings["inputSettings"]["color"] == 0xFFFFFFFF
        status = await request("GetRecordStatus")
        assert status["outputActive"] is False, "fixture unexpectedly started recording"
        image = await request("GetSourceScreenshot", {
            "sourceName": "Native white", "imageFormat": "png",
            "imageWidth": 64, "imageHeight": 64,
        })
        prefix, encoded = image["imageData"].split(",", 1)
        assert prefix == "data:image/png;base64"
        screenshot = runtime / "websocket-white.png"
        screenshot.write_bytes(base64.b64decode(encoded, validate=True))
        decoded = subprocess.run([
            "ffmpeg", "-v", "error", "-i", str(screenshot), "-frames:v", "1",
            "-f", "rawvideo", "-pix_fmt", "rgb24", "-",
        ], check=True, capture_output=True, timeout=15).stdout
        assert len(decoded) == 64 * 64 * 3 and min(decoded) >= 240, "wire screenshot is not white"
        await request("CreateScene", {"sceneName": "Native wire temporary"})
        event = await client.wait_for_event("SceneCreated", timeout=5)
        assert event and event["d"]["eventData"]["sceneName"] == "Native wire temporary"
        await request("RemoveScene", {"sceneName": "Native wire temporary"})
        batch = await client.request_batch([{"requestType": "GetVersion"}, {"requestType": "GetSceneList"}])
        assert all(item["requestStatus"]["code"] == 100 for item in batch["d"]["results"])
        unknown = await client.request("NativeMissingRequest")
        assert unknown["d"]["requestStatus"]["code"] == 204
        if os.environ.get("OBS_BUILD_VLC"):
            await verify_vlc(runtime, request)
        if recording:
            await request("StartRecord")
            deadline = asyncio.get_running_loop().time() + 5
            while True:
                status = await request("GetRecordStatus")
                if status["outputActive"]:
                    break
                assert asyncio.get_running_loop().time() < deadline, "wire recording did not start"
                await asyncio.sleep(0.05)
        evidence["authentication"] = "wrong password rejected; correct password accepted"
        evidence["scene_event_and_batch"] = True
        (runtime / "websocket-evidence.json").write_text(json.dumps(evidence, indent=2))
        print("Staged obs-websocket auth, requests, event, batch, recording status and white screenshot passed")
    finally:
        await client.close_conn()


async def verify_vlc(runtime: Path, request) -> None:
    clip = runtime / "vlc-white.mkv"
    subprocess.run([
        "ffmpeg", "-v", "error", "-y", "-f", "lavfi", "-i", "color=white:s=320x180:r=30",
        "-t", "6", "-c:v", "libx264", "-preset", "ultrafast", "-pix_fmt", "yuv420p", str(clip),
    ], check=True, capture_output=True, timeout=20)
    await request("CreateInput", {
        "sceneName": "Native scene", "inputName": "Native VLC", "inputKind": "vlc_source",
        "sceneItemEnabled": True,
        "inputSettings": {"playlist": [{"value": str(clip)}], "loop": True,
                          "playback_behavior": "always_play"},
    })
    deadline = asyncio.get_running_loop().time() + 15
    while True:
        status = await request("GetMediaInputStatus", {"inputName": "Native VLC"})
        if status["mediaState"] == "OBS_MEDIA_STATE_PLAYING" and status["mediaDuration"] > 0:
            break
        assert asyncio.get_running_loop().time() < deadline, status
        await asyncio.sleep(0.1)
    assert 5000 <= status["mediaDuration"] <= 6500, status
    while True:
        image = await request("GetSourceScreenshot", {
            "sourceName": "Native VLC", "imageFormat": "png", "imageWidth": 64, "imageHeight": 36,
        })
        screenshot = runtime / "vlc-white.png"
        screenshot.write_bytes(base64.b64decode(image["imageData"].split(",", 1)[1], validate=True))
        decoded = subprocess.run([
            "ffmpeg", "-v", "error", "-i", str(screenshot), "-frames:v", "1",
            "-f", "rawvideo", "-pix_fmt", "rgb24", "-",
        ], check=True, capture_output=True, timeout=15).stdout
        if len(decoded) == 64 * 36 * 3 and min(decoded) >= 240:
            break
        assert asyncio.get_running_loop().time() < deadline, "VLC did not decode a white frame"
        await asyncio.sleep(0.1)
    for action, expected in [("PAUSE", "PAUSED"), ("PLAY", "PLAYING"), ("RESTART", "PLAYING")]:
        await request("TriggerMediaInputAction", {
            "inputName": "Native VLC", "mediaAction": f"OBS_WEBSOCKET_MEDIA_INPUT_ACTION_{action}",
        })
        deadline = asyncio.get_running_loop().time() + 5
        while True:
            status = await request("GetMediaInputStatus", {"inputName": "Native VLC"})
            if status["mediaState"] == f"OBS_MEDIA_STATE_{expected}":
                break
            assert asyncio.get_running_loop().time() < deadline, status
            await asyncio.sleep(0.05)
    print("Native VLC playlist decoded white video; pause/play/restart passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--recording", action="store_true")
    arguments = parser.parse_args()
    asyncio.run(verify(arguments.runtime.resolve(), arguments.recording))
