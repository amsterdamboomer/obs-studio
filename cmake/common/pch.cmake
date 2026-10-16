# OBS CMake precompiled header module
#
# One unified precompiled header per target and language (cmake/pch/), so the
# C standard library, the libobs headers and, for UI targets, all of
# QtCore/QtGui/QtWidgets are parsed once per target instead of once per
# source. set_target_properties_obs calls obs_target_pch for every target;
# libobs-objects calls it directly.

include_guard(GLOBAL)

option(ENABLE_PCH "Compile each target against one unified precompiled header" ON)

set(_OBS_PCH_DIR "${CMAKE_CURRENT_LIST_DIR}/../pch")

# Sources that #define a macro before their first #include (_GNU_SOURCE,
# _FILE_OFFSET_BITS, WIN32_LEAN_AND_MEAN, ...). A force-included header would
# be seen before that definition, so they compile without the PCH.
set(
  _OBS_PCH_SKIP_SOURCES
  libobs/util/pipe-windows.c
  libobs/util/platform.c
  plugins/linux-v4l2/v4l2-output.c
  plugins/obs-outputs/librtmp/rtmp.c
  plugins/obs-outputs/net-if.c
  plugins/oss-audio/oss-input-freebsd.c
)

# _obs_pch_profile: pick the header set from what <target> links.
function(_obs_pch_profile target out)
  if(target STREQUAL obs-studio)
    set(${out} frontend PARENT_SCOPE)
    return()
  endif()
  if(target STREQUAL libobs-objects)
    set(${out} libobs PARENT_SCOPE)
    return()
  endif()
  get_target_property(_links ${target} LINK_LIBRARIES)
  set(_profile std)
  foreach(_link IN LISTS _links)
    if(_link MATCHES "^(Qt6?::Widgets|Qt::Widgets)$")
      set(_profile qt)
      break()
    elseif(_link MATCHES "^(OBS::libobs|libobs)$")
      set(_profile plugin)
    endif()
  endforeach()
  set(${out} ${_profile} PARENT_SCOPE)
endfunction()

# obs_target_pch: give <target> its unified precompiled header(s).
function(obs_target_pch target)
  if(NOT ENABLE_PCH)
    return()
  endif()
  get_target_property(_type ${target} TYPE)
  if(_type MATCHES "^(INTERFACE_LIBRARY|UTILITY)$")
    return()
  endif()
  get_target_property(_disabled ${target} OBS_DISABLE_PCH)
  if(_disabled)
    return()
  endif()

  get_target_property(_sources ${target} SOURCES)
  set(_has_c FALSE)
  set(_has_cxx FALSE)
  foreach(_source IN LISTS _sources)
    if(_source MATCHES "\\.c$")
      set(_has_c TRUE)
    elseif(_source MATCHES "\\.(cpp|cc|cxx)$")
      set(_has_cxx TRUE)
    endif()
  endforeach()

  _obs_pch_profile(${target} _profile)
  set(_c_header "${_OBS_PCH_DIR}/obs-c-std.h")
  set(_cxx_header "${_OBS_PCH_DIR}/obs-cxx-std.hpp")
  if(_profile STREQUAL libobs)
    set(_c_header "${_OBS_PCH_DIR}/obs-c-libobs.h")
  elseif(_profile MATCHES "^(plugin|qt|frontend)$")
    set(_c_header "${_OBS_PCH_DIR}/obs-c-plugin.h")
    set(_cxx_header "${_OBS_PCH_DIR}/obs-cxx-${_profile}.hpp")
  endif()

  if(_has_c)
    target_precompile_headers(${target} PRIVATE "$<$<COMPILE_LANGUAGE:C>:${_c_header}>")
  endif()
  if(_has_cxx)
    target_precompile_headers(${target} PRIVATE "$<$<COMPILE_LANGUAGE:CXX>:${_cxx_header}>")
  endif()

  get_target_property(_source_dir ${target} SOURCE_DIR)
  foreach(_source IN LISTS _sources)
    cmake_path(ABSOLUTE_PATH _source BASE_DIRECTORY "${_source_dir}" NORMALIZE OUTPUT_VARIABLE _absolute)
    foreach(_skip IN LISTS _OBS_PCH_SKIP_SOURCES)
      if(_absolute MATCHES "/${_skip}$")
        set_source_files_properties(${_source} TARGET_DIRECTORY ${target} PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
      endif()
    endforeach()
  endforeach()
endfunction()

# obs_target_pch_deferred: apply the PCH and unity settings at the end of the
# current directory, after the target's platform files added their sources
# and links.
function(obs_target_pch_deferred target)
  cmake_language(EVAL CODE "cmake_language(DEFER CALL _obs_target_build_speed [[${target}]])")
endfunction()

function(_obs_target_build_speed target)
  obs_target_pch(${target})
  obs_target_unity(${target})
endfunction()
