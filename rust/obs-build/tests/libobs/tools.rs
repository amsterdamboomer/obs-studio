//! Test-only module that exercises the actual frontend Tools actions/dialogs.
use obs_build::{NativeBuild, SharedLibrary, plugins};
use std::path::Path;

pub fn build(build: &NativeBuild, root: &Path, core: &Path, api: &Path) -> plugins::Module {
    let source = build.output.join("frontend-tools-probe.cpp");
    if std::fs::read_to_string(&source).ok().as_deref() != Some(SOURCE) {
        std::fs::write(&source, SOURCE).unwrap();
    }
    let mut target = SharedLibrary {
        name: "native-frontend-probe".into(),
        sources: vec![source],
        file_name: Some("native-frontend-probe.so".into()),
        include_dirs: vec![
            root.into(),
            root.join("libobs"),
            root.join("frontend/api"),
            build.output.join("config"),
        ],
        dependencies: vec![core.into(), api.into()],
        cpp_compile_options: vec![
            "-std=gnu++17".into(),
            "-fvisibility=hidden".into(),
            "-fvisibility-inlines-hidden".into(),
        ],
        ..SharedLibrary::default()
    };
    for package in ["simde", "Qt6Widgets"] {
        let dependency = pkg_config::Config::new()
            .cargo_metadata(false)
            .probe(package)
            .unwrap();
        target.include_dirs.extend(dependency.include_paths);
        let mut definitions: Vec<_> = dependency.defines.into_iter().collect();
        definitions.sort();
        target.definitions.extend(definitions);
        for path in dependency.link_paths {
            target.link_options.extend([
                format!("-L{}", path.display()),
                format!("-Wl,-rpath,{}", path.display()),
            ]);
        }
        target
            .link_options
            .extend(dependency.libs.iter().map(|name| format!("-l{name}")));
    }
    let data = build.output.join("frontend-tools-probe-data");
    std::fs::create_dir_all(&data).unwrap();
    plugins::Module {
        library: build.shared_library(&target).unwrap(),
        data,
    }
}

const SOURCE: &str = r#"
#include <obs-module.h>
#include <obs-frontend-api.h>
#include <QApplication>
#include <QMainWindow>
#include <QMenuBar>
#include <QMenu>
#include <QAction>
#include <QDialog>
#include <QListWidget>
#include <QSpinBox>
#include <QLabel>
#include <QTimer>
#include <QEventLoop>
#include <QPointer>
#include <QPixmap>
#include <cstdlib>

OBS_DECLARE_MODULE()

static bool open_dialog(QMenu *menu, const char *label, const char *type, const char *image, int scripts = -1)
{
    QAction *selected = nullptr;
    for (QAction *action : menu->actions()) {
        QString text = action->text(); text.remove('&');
        if (text == label) { selected = action; break; }
    }
    if (!selected) { blog(LOG_ERROR, "cc::Build missing Tools action %s", label); return false; }
    // SceneSwitcher runs a nested modal event loop and is destroyed before
    // trigger() returns. OutputTimer instead queues its show() after 250 ms.
    // Inspect and close both while their event loops are running.
    QObject context;
    QEventLoop loop;
    bool completed = false;
    bool passed = false;
    QTimer::singleShot(400, &context, [&]() {
        QPointer<QWidget> dialog;
        for (QWidget *widget : QApplication::topLevelWidgets())
            if (widget->inherits(type) && widget->isVisible()) { dialog = widget; break; }
        if (!dialog) {
            blog(LOG_ERROR, "cc::Build Tools dialog %s did not open", type);
        } else {
            bool valid = true;
            if (scripts >= 0) {
                QListWidget *list = dialog->findChild<QListWidget *>("scripts");
                valid = list && list->count() == scripts;
                if (!valid) blog(LOG_ERROR, "cc::Build incorrect saved script count");
                if (valid) {
                    for (int row = 0; row < scripts; ++row) {
                        list->setCurrentRow(row);
                        // OBSPropertiesView queues its initial property reload.
                        QApplication::processEvents();
                        QLabel *description = dialog->findChild<QLabel *>("description");
                        const auto fields = dialog->findChildren<QSpinBox *>();
                        bool seed = false;
                        for (QSpinBox *field : fields) seed = seed || field->value() == 41;
                        if (!description || !description->text().startsWith("Native GUI ") || !seed) {
                            blog(LOG_ERROR, "cc::Build saved script properties were not displayed for row %d", row);
                            valid = false;
                        }
                    }
                    if (scripts) {
                        list->setCurrentRow(0);
                        QApplication::processEvents();
                    }
                }
            }
            const char *directory = std::getenv("OBS_BUILD_GUI_EVIDENCE");
            bool saved = directory && dialog->grab().save(QString::fromUtf8(directory) + "/" + image);
            bool closed = dialog->close();
            passed = valid && saved && closed && (!dialog || !dialog->isVisible());
            if (!passed) blog(LOG_ERROR, "cc::Build Tools inspection/screenshot/close failed");
        }
        completed = true;
        loop.quit();
    });
    selected->trigger();
    if (!completed) loop.exec();
    return passed;
}

static void verify_dialogs()
{
    auto *main = static_cast<QMainWindow *>(obs_frontend_get_main_window());
    if (!main) { blog(LOG_ERROR, "cc::Build main window missing"); return; }
    QMenu *tools = nullptr;
    for (QAction *action : main->menuBar()->actions()) {
        QString text = action->text(); text.remove('&');
        if (text == "Tools") tools = action->menu();
    }
    if (!tools) { blog(LOG_ERROR, "cc::Build Tools menu missing"); return; }
    if (!open_dialog(tools, "Automatic Scene Switcher", "SceneSwitcher", "tools-scene-switcher.png")) return;
    if (!open_dialog(tools, "Output Timer", "OutputTimer", "tools-output-timer.png")) return;
    const char *count = std::getenv("OBS_BUILD_GUI_SCRIPT_COUNT");
    if (count && std::atoi(count) >= 0 && !open_dialog(tools, "Scripts", "ScriptsTool", "tools-scripts.png", std::atoi(count))) return;
    if (!count || std::atoi(count) < 0) {
        for (QAction *action : tools->actions()) {
            QString text = action->text(); text.remove('&');
            if (text == "Scripts") { blog(LOG_ERROR, "cc::Build disabled Scripts action is present"); return; }
        }
    }
    blog(LOG_INFO, "cc::Build frontend Tools dialogs verified");
}

static bool frontend_exited = false;

static void frontend_event(enum obs_frontend_event event, void *)
{
    if (event == OBS_FRONTEND_EVENT_EXIT) frontend_exited = true;
    if (event == OBS_FRONTEND_EVENT_FINISHED_LOADING)
        QTimer::singleShot(0, verify_dialogs);
}

bool obs_module_load(void)
{
    obs_frontend_add_event_callback(frontend_event, nullptr);
    return true;
}
void obs_module_unload(void) { if (!frontend_exited) obs_frontend_remove_event_callback(frontend_event, nullptr); }
"#;
