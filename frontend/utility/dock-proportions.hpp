#pragma once

/* Keeps the relative sizes of docks that share a row in the main window
 * when the window shrinks below the docks' minimum sizes and grows again
 * (obs-rust/obs-studio#2225). Header-only, Qt Widgets only, so it can be
 * unit tested without the frontend. */

#include <QMainWindow>
#include <QObject>

class DockProportions : public QObject {
public:
	explicit DockProportions(QMainWindow *window) : QObject(window) {}
};
