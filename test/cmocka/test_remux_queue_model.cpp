/* Tools > Remux: the queue model behind the remux dialog's table.
 *
 * beginProcessing() and endProcessing() hide and show the empty insertion row
 * at the end of the queue. They sent rowsRemoved/rowsInserted while
 * rowCount() still gave the old count, so a view or proxy saw a row count
 * that disagreed with the notification. Their dataChanged and the one from
 * checkInputPath() also ended one past the last row or column, so the
 * bottom-right index was invalid (and with an empty queue, the top-left too).
 *
 * Builds the real utility/RemuxQueueModel.cpp and checks every notification
 * against rowCount()/columnCount(). Qt's offscreen platform, no display. */

#include <QApplication>
#include <QModelIndex>
#include "frontend/utility/RemuxQueueModel.hpp"

#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <stdint.h>
extern "C" {
#include <cmocka.h>
}

namespace {

/* Records whether every notification the model sends is consistent. */
struct Watcher {
	RemuxQueueModel &model;
	int badDataChanged = 0;
	int dataChanged = 0;
	int rowsRemovedCount = -1;
	int rowsInsertedCount = -1;
	int aboutToRemoveCount = -1;
	int aboutToInsertCount = -1;

	explicit Watcher(RemuxQueueModel &m) : model(m)
	{
		QObject::connect(&model, &QAbstractItemModel::dataChanged,
				 [this](const QModelIndex &tl, const QModelIndex &br) {
					 dataChanged++;
					 if (!tl.isValid() || !br.isValid() || tl.row() > br.row() ||
					     tl.column() > br.column() || br.row() >= model.rowCount() ||
					     br.column() >= model.columnCount()) {
						 badDataChanged++;
					 }
				 });
		QObject::connect(&model, &QAbstractItemModel::rowsAboutToBeRemoved,
				 [this](const QModelIndex &, int, int) { aboutToRemoveCount = model.rowCount(); });
		QObject::connect(&model, &QAbstractItemModel::rowsAboutToBeInserted,
				 [this](const QModelIndex &, int, int) { aboutToInsertCount = model.rowCount(); });
		QObject::connect(&model, &QAbstractItemModel::rowsRemoved,
				 [this](const QModelIndex &, int, int) { rowsRemovedCount = model.rowCount(); });
		QObject::connect(&model, &QAbstractItemModel::rowsInserted,
				 [this](const QModelIndex &, int, int) { rowsInsertedCount = model.rowCount(); });
	}
};

void addEntry(RemuxQueueModel &model, const char *path)
{
	int row = model.rowCount() - 1; /* the insertion row */
	assert_true(model.setData(model.index(row, RemuxEntryColumn::InputPath), QString(path), Qt::EditRole));
}

} // namespace

/* Editing an input path reports the changed row's cells, all of them valid. */
static void input_path_change_range_is_valid(void **)
{
	RemuxQueueModel model;
	addEntry(model, "/nonexistent/a.flv");
	Watcher w(model);

	assert_true(model.setData(model.index(0, RemuxEntryColumn::InputPath), QString("/nonexistent/b.mkv"),
				  Qt::EditRole));
	assert_int_equal(w.dataChanged, 1);
	assert_int_equal(w.badDataChanged, 0);
}

/* Starting: rowCount() still has the insertion row when rowsAboutToBeRemoved
 * arrives and already lacks the insertion row when rowsRemoved
 * arrives, and the state column update covers only real rows. */
static void begin_processing_notifies_consistently(void **)
{
	RemuxQueueModel model;
	addEntry(model, "/nonexistent/a.flv");
	addEntry(model, "/nonexistent/b.flv");
	assert_int_equal(model.rowCount(), 3);
	Watcher w(model);

	model.beginProcessing();
	assert_int_equal(model.rowCount(), 2);
	assert_int_equal(w.aboutToRemoveCount, 3);
	assert_int_equal(w.rowsRemovedCount, 2);
	assert_int_equal(w.badDataChanged, 0);
}

/* Finishing: rowCount() still lacks the insertion row when
 * rowsAboutToBeInserted arrives and already has the insertion row back when
 * rowsInserted arrives, and the state column update stays in range. */
static void end_processing_notifies_consistently(void **)
{
	RemuxQueueModel model;
	addEntry(model, "/nonexistent/a.flv");
	model.beginProcessing();
	Watcher w(model);

	model.endProcessing();
	assert_int_equal(model.rowCount(), 2);
	assert_int_equal(w.aboutToInsertCount, 1);
	assert_int_equal(w.rowsInsertedCount, 2);
	assert_int_equal(w.badDataChanged, 0);
}

/* An empty queue has no real rows to update: no invalid range is sent. */
static void empty_queue_sends_no_invalid_range(void **)
{
	RemuxQueueModel model;
	Watcher w(model);

	model.beginProcessing();
	assert_int_equal(w.rowsRemovedCount, 0);
	model.endProcessing();
	assert_int_equal(w.rowsInsertedCount, 1);
	assert_int_equal(w.badDataChanged, 0);
}

int main(int argc, char **argv)
{
	qputenv("QT_QPA_PLATFORM", "offscreen");
	QApplication app(argc, argv);

	const struct CMUnitTest tests[] = {
		cmocka_unit_test(input_path_change_range_is_valid),
		cmocka_unit_test(begin_processing_notifies_consistently),
		cmocka_unit_test(end_processing_notifies_consistently),
		cmocka_unit_test(empty_queue_sends_no_invalid_range),
	};
	return cmocka_run_group_tests(tests, NULL, NULL);
}
