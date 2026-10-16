# OBS CMake unity build module
#
# With ENABLE_UNITY_BUILD, each target compiles its sources in batches of
# OBS_UNITY_BUILD_BATCH_SIZE concatenated translation units. File-local names
# (static functions and variables, file-scope macros) must then be unique
# across a target; targets not yet cleaned set OBS_DISABLE_UNITY_BUILD.
# obs_target_unity is called with obs_target_pch from set_target_properties_obs.

include_guard(GLOBAL)

option(ENABLE_UNITY_BUILD "Compile each target as batches of concatenated sources" OFF)
set(OBS_UNITY_BUILD_BATCH_SIZE 8 CACHE STRING "Sources per unity translation unit")

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
    PROPERTIES
      UNITY_BUILD ON
      UNITY_BUILD_BATCH_SIZE ${OBS_UNITY_BUILD_BATCH_SIZE}
      UNITY_BUILD_UNIQUE_ID OBS_UNITY_ID
  )

  # The PCH-exempt sources define macros before their includes, so they must
  # be the first thing their translation unit sees: keep them out of batches.
  get_target_property(_sources ${target} SOURCES)
  get_target_property(_source_dir ${target} SOURCE_DIR)
  foreach(_source IN LISTS _sources)
    cmake_path(ABSOLUTE_PATH _source BASE_DIRECTORY "${_source_dir}" NORMALIZE OUTPUT_VARIABLE _absolute)
    foreach(_skip IN LISTS _OBS_PCH_SKIP_SOURCES)
      if(_absolute MATCHES "/${_skip}$")
        set_source_files_properties(${_source} TARGET_DIRECTORY ${target} PROPERTIES SKIP_UNITY_BUILD_INCLUSION ON)
      endif()
    endforeach()
  endforeach()
endfunction()
