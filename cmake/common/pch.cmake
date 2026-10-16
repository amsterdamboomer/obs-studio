# OBS CMake precompiled header module
#
# One unified precompiled header per target and language (cmake/pch/), so the
# C standard library, the libobs headers and, for UI targets, all of
# QtCore/QtGui/QtWidgets are parsed once per target instead of once per
# source. Applied to every first-party target at the end of the top-level
# CMakeLists.txt (_obs_build_speed_all).

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
  set(_qt FALSE)
  set(_obs FALSE)
  foreach(_link IN LISTS _links)
    if(_link MATCHES "^(Qt6?::Widgets|Qt::Widgets)$")
      set(_qt TRUE)
    elseif(_link MATCHES "^(OBS::libobs|libobs)$")
      set(_obs TRUE)
    endif()
  endforeach()
  if(_qt AND _obs)
    set(${out} qt PARENT_SCOPE)
  elseif(_qt)
    set(${out} qt-std PARENT_SCOPE)
  elseif(_obs)
    set(${out} plugin PARENT_SCOPE)
  else()
    set(${out} std PARENT_SCOPE)
  endif()
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
  elseif(_profile STREQUAL qt-std)
    set(_cxx_header "${_OBS_PCH_DIR}/obs-cxx-qt-std.hpp")
  endif()

  if(_has_c)
    target_precompile_headers(${target} PRIVATE "$<$<COMPILE_LANGUAGE:C>:${_c_header}>")
  endif()
  if(_has_cxx)
    target_precompile_headers(${target} PRIVATE "$<$<COMPILE_LANGUAGE:CXX>:${_cxx_header}>")
  endif()

  _obs_target_sources_matching(${target} _skipped ${_OBS_PCH_SKIP_SOURCES})
  if(_skipped)
    set_source_files_properties(${_skipped} TARGET_DIRECTORY ${target} PROPERTIES SKIP_PRECOMPILE_HEADERS ON)
  endif()
endfunction()

# _obs_target_sources_matching: absolute paths of <target>'s sources that end
# in one of the given repository-relative paths. Sources added through
# generator expressions ($<$<BOOL:...>:file.c>) are matched by the file names
# inside them.
function(_obs_target_sources_matching target out)
  get_target_property(_sources ${target} SOURCES)
  get_target_property(_source_dir ${target} SOURCE_DIR)
  set(_matches "")
  foreach(_source IN LISTS _sources)
    string(REGEX MATCHALL "[^:<>$,;]+\\.(c|cpp|cc|m|mm)" _files "${_source}")
    foreach(_file IN LISTS _files)
      cmake_path(ABSOLUTE_PATH _file BASE_DIRECTORY "${_source_dir}" NORMALIZE OUTPUT_VARIABLE _absolute)
      foreach(_wanted IN LISTS ARGN)
        if(_absolute MATCHES "/${_wanted}$")
          list(APPEND _matches "${_absolute}")
        endif()
      endforeach()
    endforeach()
  endforeach()
  set(${out} ${_matches} PARENT_SCOPE)
endfunction()

# _obs_target_build_speed: the PCH and unity settings for one target.
function(_obs_target_build_speed target)
  get_target_property(_imported ${target} IMPORTED)
  get_target_property(_source_dir ${target} SOURCE_DIR)
  if(_imported OR NOT _source_dir)
    return()
  endif()
  # Third-party code (deps/), the unchanged C tests (test/) and Corrosion's
  # Cargo targets keep their own build.
  cmake_path(RELATIVE_PATH _source_dir BASE_DIRECTORY "${CMAKE_SOURCE_DIR}" OUTPUT_VARIABLE _relative)
  if(_relative MATCHES "^(deps|test|rust)(/|$)" OR target MATCHES "^(_cargo|cargo-|corrosion)")
    return()
  endif()
  obs_target_pch(${target})
  obs_target_unity(${target})
endfunction()

# _obs_build_speed_all: apply the settings to every target of <directory> and
# its subdirectories. Runs once, deferred to the end of the top-level
# CMakeLists.txt, when every target and its sources exist.
function(_obs_build_speed_all directory)
  get_property(_targets DIRECTORY "${directory}" PROPERTY BUILDSYSTEM_TARGETS)
  foreach(_target IN LISTS _targets)
    _obs_target_build_speed(${_target})
  endforeach()
  get_property(_subdirectories DIRECTORY "${directory}" PROPERTY SUBDIRECTORIES)
  foreach(_subdirectory IN LISTS _subdirectories)
    _obs_build_speed_all("${_subdirectory}")
  endforeach()
endfunction()

if(NOT _OBS_BUILD_SPEED_DEFERRED)
  set(_OBS_BUILD_SPEED_DEFERRED TRUE)
  cmake_language(DEFER DIRECTORY "${CMAKE_SOURCE_DIR}" CALL _obs_build_speed_all "${CMAKE_SOURCE_DIR}")
endif()
