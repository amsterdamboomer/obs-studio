/* Precompiled header: Qt UI targets (cmake/common/pch.cmake): the whole of
 * QtCore, QtGui and QtWidgets, compiled once per target instead of once per
 * source.
 *
 * Qt comes before the libobs headers: libobs's graphics headers map x86
 * intrinsics through SIMDe's native aliases, which break Qt's own
 * <smmintrin.h> include if they are seen first. */

#pragma once

#include "obs-cxx-qt-std.hpp"

#include <util/c99defs.h>
#include <util/base.h>
#include <util/bmem.h>
#include <util/darray.h>
#include <util/dstr.h>
#include <util/platform.h>
#include <util/threading.h>
#include <obs-module.h>
#include <obs.hpp>
#include <util/util.hpp>
