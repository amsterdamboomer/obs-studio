# OBS CMake unity build module
#
# With ENABLE_UNITY_BUILD, each target compiles its sources in batches of
# OBS_UNITY_BUILD_BATCH_SIZE concatenated translation units. File-local names
# (static functions and variables, file-scope macros) must then be unique
# across a target; targets not yet cleaned set OBS_DISABLE_UNITY_BUILD.
# obs_target_unity runs with obs_target_pch for every first-party target.

include_guard(GLOBAL)

# On by default where every target's file-local names are unique (Linux);
# Windows and macOS sources still have collisions to rename.
if(OS_LINUX)
  option(ENABLE_UNITY_BUILD "Compile each target as batches of concatenated sources" ON)
else()
  option(ENABLE_UNITY_BUILD "Compile each target as batches of concatenated sources" OFF)
endif()
set(OBS_UNITY_BUILD_BATCH_SIZE 8 CACHE STRING "Sources per unity translation unit")

# Sources that duplicate another source of the same target wholesale (an
# alternative backend, a kept legacy version): they compile on their own
# rather than being renamed symbol by symbol.
set(_OBS_UNITY_SKIP_SOURCES libobs-opengl/gl-wayland-egl.c plugins/image-source/obs-slideshow.c)

# obs_target_unity: turn on unity builds for <target>.
function(obs_target_unity target)
  if(NOT ENABLE_UNITY_BUILD)
    return()
  endif()
  get_target_property(_type ${target} TYPE)
  if(_type MATCHES "^(INTERFACE_LIBRARY|UTILITY)$")
    return()
  endif()
  get_target_property(_disabled ${target} OBS_DISABLE_UNITY_BUILD)
  if(_disabled)
    return()
  endif()

  set_target_properties(
    ${target}
    PROPERTIES UNITY_BUILD ON UNITY_BUILD_BATCH_SIZE ${OBS_UNITY_BUILD_BATCH_SIZE} UNITY_BUILD_UNIQUE_ID OBS_UNITY_ID
  )

  # The PCH-exempt sources define macros before their includes, so they must
  # be the first thing their translation unit sees; the duplicates above
  # can't share one. Keep both out of batches.
  _obs_target_sources_matching(${target} _skipped ${_OBS_PCH_SKIP_SOURCES} ${_OBS_UNITY_SKIP_SOURCES})
  if(_skipped)
    set_source_files_properties(${_skipped} TARGET_DIRECTORY ${target} PROPERTIES SKIP_UNITY_BUILD_INCLUSION ON)
  endif()
endfunction()
