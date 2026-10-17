#include "frontend/build/frontend-utility-disk-space-monitor.pch.hpp"

/* RED stub: queries synchronously on the caller's thread. */
struct DiskSpaceMonitor::State {
	DiskSpaceMonitor::FreeSpaceFn free_space = nullptr;
	std::string path;
	DiskSpaceSample sample;
	uint64_t started = 0;
};

DiskSpaceMonitor::DiskSpaceMonitor(FreeSpaceFn free_space) : state(std::make_shared<State>())
{
	state->free_space = free_space ? free_space : os_get_free_disk_space;
}

DiskSpaceMonitor::~DiskSpaceMonitor() {}

void DiskSpaceMonitor::request(const std::string &path)
{
	state->started++;
	state->path = path;
	state->sample.bytes = state->free_space(path.c_str());
	state->sample.valid = true;
}

DiskSpaceSample DiskSpaceMonitor::latest(const std::string &path) const
{
	return path == state->path ? state->sample : DiskSpaceSample{};
}

uint64_t DiskSpaceMonitor::queries_started() const
{
	return state->started;
}
