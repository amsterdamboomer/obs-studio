//! Uses the unchanged C headers for graphics/video structs and callbacks.
use obs_build::{NativeBuild, SharedLibrary};
use std::path::{Path, PathBuf};

pub fn build(build: &NativeBuild, root: &Path, core: &Path) -> PathBuf {
    std::fs::create_dir_all(&build.output).unwrap();
    let source = build.output.join("graphics-probe.c");
    if std::fs::read_to_string(&source).ok().as_deref() != Some(SOURCE) {
        std::fs::write(&source, SOURCE).unwrap();
    }
    let mut library = SharedLibrary {
        name: "graphics-probe".into(),
        sources: vec![source],
        include_dirs: vec![
            root.into(),
            root.join("libobs"),
            build.output.join("config"),
        ],
        dependencies: vec![core.into()],
        compile_options: vec!["-std=gnu17".into()],
        ..SharedLibrary::default()
    };
    for package in ["simde", "x11"] {
        let dependency = pkg_config::Config::new()
            .cargo_metadata(false)
            .probe(package)
            .unwrap();
        library.include_dirs.extend(dependency.include_paths);
        for path in dependency.link_paths {
            library.link_options.push(format!("-L{}", path.display()));
        }
        library
            .link_options
            .extend(dependency.libs.iter().map(|name| format!("-l{name}")));
    }
    build.shared_library(&library).unwrap()
}

const SOURCE: &str = r#"
#include <X11/Xlib.h>
#include <stdatomic.h>
#include <obs.h>
#include <obs-nix-platform.h>
#include <util/platform.h>

static Display *test_display;
static atomic_uint frames;
static atomic_uint invalid_frames;

EXPORT bool obs_fixture_open_display(void)
{
    XInitThreads();
    test_display = XOpenDisplay(NULL);
    if (!test_display)
        return false;
    obs_set_nix_platform(OBS_NIX_PLATFORM_X11_EGL);
    obs_set_nix_platform_display(test_display);
    return true;
}

static void receive_frame(void *unused, struct video_data *frame)
{
    (void)unused;
    if (!frame->data[0] || frame->linesize[0] < 64 || frame->data[0][0] != 16)
        atomic_fetch_add(&invalid_frames, 1);
    atomic_fetch_add(&frames, 1);
}

EXPORT int obs_fixture_render(const char *renderer, const char *data_path)
{
    obs_add_data_path(data_path);
    struct obs_video_info video = {
        .graphics_module = renderer,
        .fps_num = 30, .fps_den = 1,
        .base_width = 64, .base_height = 64,
        .output_width = 64, .output_height = 64,
        .output_format = VIDEO_FORMAT_NV12,
        .gpu_conversion = true,
        .colorspace = VIDEO_CS_709,
        .range = VIDEO_RANGE_PARTIAL,
        .scale_type = OBS_SCALE_BICUBIC,
    };
    int result = obs_reset_video(&video);
    if (result != OBS_VIDEO_SUCCESS)
        return result;
    obs_enter_graphics();
    blog(LOG_INFO, "cc::Build graphics device: %s", gs_get_device_name());
    obs_leave_graphics();
    obs_scene_t *scene = obs_scene_create("cc-build-rendering");
    if (!scene)
        return 81;
    obs_set_output_source(0, obs_scene_get_source(scene));
    atomic_store(&frames, 0);
    atomic_store(&invalid_frames, 0);
    obs_add_raw_video_callback(NULL, receive_frame, NULL);
    for (unsigned i = 0; i < 200 && atomic_load(&frames) < 3; ++i)
        os_sleep_ms(10);
    obs_remove_raw_video_callback(receive_frame, NULL);
    obs_set_output_source(0, NULL);
    obs_scene_release(scene);
    /* The owning test calls obs_shutdown after this probe. */
    if (atomic_load(&frames) < 3)
        return 82;
    return atomic_load(&invalid_frames) ? 83 : 0;
}

EXPORT void obs_fixture_close_display(void)
{
    obs_set_nix_platform_display(NULL);
    XCloseDisplay(test_display);
    test_display = NULL;
}
"#;
