project(obs-cmocka)

find_package(CMocka CONFIG REQUIRED)
add_library(obs-native-test-cmocka INTERFACE)
add_library(OBS::native-test-cmocka ALIAS obs-native-test-cmocka)
target_include_directories(obs-native-test-cmocka INTERFACE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(obs-native-test-cmocka INTERFACE ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  find_package(Threads REQUIRED)
endif()

# Every test executable joins this target, so the Tier 2 validators
# (rust/tools/*-validate) build `cmocka-tests` instead of a hand-kept list and a
# new test cannot go unbuilt (and show up in ctest as "Not Run").
add_custom_target(cmocka-tests)

# The display tests' add_test commands wrap themselves in xvfb-run, found as
# XVFB_RUN, XVFB_RUN_PROGRAM or XVFB_RUN_EXECUTABLE. Point all three at
# xvfb-run-or-display.sh, which reuses the display ctest already runs under
# instead of every test racing to start its own server (the fragments'
# find_program keeps a variable that is already set).
find_program(OBS_XVFB_RUN_REAL xvfb-run)
if(OBS_XVFB_RUN_REAL)
  configure_file(xvfb-run-or-display.sh.in "${CMAKE_CURRENT_BINARY_DIR}/xvfb-run-or-display.sh" @ONLY)
  file(
    CHMOD
    "${CMAKE_CURRENT_BINARY_DIR}/xvfb-run-or-display.sh"
    PERMISSIONS OWNER_READ OWNER_WRITE OWNER_EXECUTE GROUP_READ GROUP_EXECUTE WORLD_READ WORLD_EXECUTE
  )
  foreach(_xvfb_var IN ITEMS XVFB_RUN XVFB_RUN_PROGRAM XVFB_RUN_EXECUTABLE)
    set(${_xvfb_var} "${CMAKE_CURRENT_BINARY_DIR}/xvfb-run-or-display.sh" CACHE FILEPATH "" FORCE)
  endforeach()
endif()

# Existing C tests are never edited to satisfy a new warning, and they pass
# float to double varargs and math freely (as does the SWIG-generated code some
# of them compile), so -Wdouble-promotion (obs-rust/obs-studio#516) covers the
# product code, not the tests.
add_compile_options($<$<COMPILE_LANG_AND_ID:C,AppleClang,Clang,GNU>:-Wno-double-promotion>)
# Existing C tests are never edited to satisfy a new warning, and they cast
# const away freely, so -Wcast-qual (obs-rust/obs-studio#516) covers the
# product code, not the tests.
add_compile_options($<$<COMPILE_LANG_AND_ID:C,AppleClang,Clang,GNU>:-Wno-cast-qual>)
# Likewise -Wformat=2 (obs-rust/obs-studio#516) covers the product code only.
add_compile_options($<$<COMPILE_LANG_AND_ID:C,AppleClang,Clang,GNU>:-Wno-format-nonliteral>)
# Existing C tests are never edited (obs-rust/obs-studio#516).
add_compile_options($<$<COMPILE_LANG_AND_ID:C,AppleClang,Clang,GNU>:-Wno-shadow>)

# Per-test sanitizers (obs-rust/obs-studio#159). Some tests compile the code
# they check with ASan/UBSan so a bug fails the test. Those flags are added
# only here: never on MSVC, never when the whole build is already sanitized
# (mixing -fsanitize=thread with address does not build, and valgrind can't
# run ASan binaries), and not when OBS_TEST_SANITIZE_PER_TARGET is OFF.
# ci/test_cmocka_sanitize.py rejects raw -fsanitize flags anywhere else.
option(OBS_TEST_SANITIZE_PER_TARGET "Build selected cmocka tests with their own ASan/UBSan" ON)
function(obs_test_sanitize target sanitizers)
  if(MSVC OR NOT OBS_TEST_SANITIZE_PER_TARGET)
    return()
  endif()
  if("${CMAKE_C_FLAGS} ${CMAKE_CXX_FLAGS} ${CMAKE_EXE_LINKER_FLAGS}" MATCHES "-fsanitize=")
    return()
  endif()
  set(compile_options -fsanitize=${sanitizers} -fno-omit-frame-pointer)
  if(sanitizers MATCHES "undefined")
    list(APPEND compile_options -fno-sanitize-recover=undefined)
  endif()
  target_compile_options(${target} PRIVATE ${compile_options})
  target_link_options(${target} PRIVATE -fsanitize=${sanitizers})
endfunction()

# Serializer test
# test_serializer is defined in build/test-graph.json.

# darray test
# test_darray is defined in build/test-graph.json.

# bitstream test
# test_bitstream is defined in build/test-graph.json.

# OS path test
# test_os_path is defined in build/test-graph.json.

# Product name shown by the frontend (frontend/utility/product-name.c), built
# without Qt.
add_executable(test_product_name test_product_name.c "${CMAKE_SOURCE_DIR}/frontend/utility/product-name.c.h")
target_include_directories(test_product_name PRIVATE ${CMOCKA_INCLUDE_DIR} "${CMAKE_SOURCE_DIR}/frontend/utility")
target_link_libraries(test_product_name PRIVATE ${CMOCKA_LIBRARIES})
add_dependencies(cmocka-tests test_product_name)
add_test(test_product_name ${CMAKE_CURRENT_BINARY_DIR}/test_product_name)

# Exercise hotkey loading with an isolated core and no platform input thread.
add_executable(
  test_hotkey_load
  test_hotkey_load.c
  "${CMAKE_SOURCE_DIR}/libobs/obs-hotkey.c.h"
  "${CMAKE_SOURCE_DIR}/libobs/obs-hotkey-bindings.c.h"
  "${CMAKE_SOURCE_DIR}/libobs/obs-hotkey-translations.c.h"
  "${CMAKE_SOURCE_DIR}/libobs/obs-hotkey-press.c.h"
  "${CMAKE_SOURCE_DIR}/libobs/obs-hotkey-pair.c.h"
)
target_compile_definitions(test_hotkey_load PRIVATE obs=hotkey_test_obs)
target_include_directories(test_hotkey_load PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_hotkey_load PRIVATE OBS::libobs OBS::caption ${CMOCKA_LIBRARIES})
add_dependencies(cmocka-tests test_hotkey_load)
add_test(test_hotkey_load ${CMAKE_CURRENT_BINARY_DIR}/test_hotkey_load)

# A core module that cannot be loaded (its file is missing, or a library it
# needs, like libcef.so for obs-browser, is not installed) must not stop OBS
# from loading the others or from starting (obs-rust/obs-studio#404). The test
# lays out stub modules under every core module name next to a copy of itself.
if(CMAKE_SYSTEM_NAME STREQUAL "Linux" AND TARGET OBS::libobs-core-modules)
  add_library(core_module_stub_dep SHARED core_module_stub_dep.c)
  set_target_properties(core_module_stub_dep PROPERTIES OUTPUT_NAME obs-test-missing-dep)

  add_library(core_module_stub MODULE core_module_stub.c)
  target_link_libraries(core_module_stub PRIVATE OBS::libobs)
  set_target_properties(core_module_stub PROPERTIES PREFIX "")

  # No RPATH: the dependency is out of reach at run time, as libcef.so was.
  add_library(core_module_stub_needs_dep MODULE core_module_stub.c)
  target_compile_definitions(core_module_stub_needs_dep PRIVATE CORE_STUB_NEEDS_DEP)
  target_link_libraries(core_module_stub_needs_dep PRIVATE OBS::libobs core_module_stub_dep)
  set_target_properties(core_module_stub_needs_dep PROPERTIES PREFIX "" SKIP_BUILD_RPATH TRUE)

  add_executable(test_core_module_load test_core_module_load.c)
  target_include_directories(test_core_module_load PRIVATE ${CMOCKA_INCLUDE_DIR})
  target_compile_definitions(
    test_core_module_load
    PRIVATE
      CORE_STUB_PATH="$<TARGET_FILE:core_module_stub>"
      CORE_STUB_NEEDS_DEP_PATH="$<TARGET_FILE:core_module_stub_needs_dep>"
  )
  target_link_libraries(test_core_module_load PRIVATE OBS::libobs OBS::libobs-core-modules ${CMOCKA_LIBRARIES})
  add_dependencies(test_core_module_load core_module_stub core_module_stub_needs_dep)
  add_dependencies(cmocka-tests test_core_module_load)
  add_test(test_core_module_load ${CMAKE_CURRENT_BINARY_DIR}/test_core_module_load)
  set_tests_properties(test_core_module_load PROPERTIES SKIP_RETURN_CODE 77 TIMEOUT 60)
endif()

# /dev/full reproduces delayed disk-write errors without fault injection.
if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
  # test_config_save is defined in build/test-graph.json.

  # test_file_serializer_safe is defined in build/test-graph.json.

  # test_quick_write is defined in build/test-graph.json.
endif()

# AV1 OBU helpers (obs-av1.c)
# test_av1 is defined in build/test-graph.json.

# NAL start-code search (obs-nal.c)
# test_nal is defined in build/test-graph.json.

# AVC header test. Where the toolchain supports it, obs-avc.c is compiled into
# the test with UBSan (no recovery) so undefined behavior fails the test
# (obs-rust/obs-studio#11); its definitions take precedence over libobs's.
add_executable(test_avc test_avc.c)
target_include_directories(test_avc PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_avc PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_sources(test_avc PRIVATE "${CMAKE_SOURCE_DIR}/libobs/obs-avc.c.h")
  obs_test_sanitize(test_avc undefined)
endif()

add_dependencies(cmocka-tests test_avc)
add_test(test_avc ${CMAKE_CURRENT_BINARY_DIR}/test_avc)

# AVC helpers through libobs only, so ENABLE_RUST_LIBOBS=ON tests the port
# (test_avc above compiles its own obs-avc.c where UBSan is available).
# test_avc_parse is defined in build/test-graph.json.

# HEVC helpers (obs-hevc.c, built only with ENABLE_HEVC)
if(ENABLE_HEVC)
  # test_hevc is defined in build/test-graph.json.
endif()

# Formatted filename test (obs-rust/obs-studio#21). platform-filename.c is
# compiled into the test so the test's obs_get_video_info and time fixtures
# replace libobs's and libc's and no obs core has to be started; its definitions
# take precedence over libobs's.
if(NOT MSVC)
  add_executable(
    test_formatted_filename
    test_formatted_filename.c
    "${CMAKE_SOURCE_DIR}/libobs/util/platform-filename.c.h"
  )
  target_include_directories(test_formatted_filename PRIVATE ${CMOCKA_INCLUDE_DIR})
  target_link_libraries(test_formatted_filename PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
  obs_test_sanitize(test_formatted_filename address,undefined)

  add_dependencies(cmocka-tests test_formatted_filename)
  add_test(test_formatted_filename ${CMAKE_CURRENT_BINARY_DIR}/test_formatted_filename)
endif()

# CRC32 test
# test_crc32 is defined in build/test-graph.json.

# UTF-8 test. utf8.c is not exported by libobs, so it is compiled into the test
# (non-Windows only: the Windows implementation wraps the Win32 API).
if(NOT MSVC)
  add_executable(test_utf8 test_utf8.c "${CMAKE_SOURCE_DIR}/libobs/util/utf8.c.h")
  target_include_directories(test_utf8 PRIVATE ${CMOCKA_INCLUDE_DIR})
  target_link_libraries(test_utf8 PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
  obs_test_sanitize(test_utf8 address,undefined)

  add_dependencies(cmocka-tests test_utf8)
  add_test(test_utf8 ${CMAKE_CURRENT_BINARY_DIR}/test_utf8)
endif()

# dstr test
# test_dstr is defined in build/test-graph.json.

# dstr regression tests (obs-rust/obs-studio#46, #47)
add_executable(test_dstr_regressions test_dstr_regressions.c)
target_include_directories(test_dstr_regressions PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_dstr_regressions PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})

add_dependencies(cmocka-tests test_dstr_regressions)
add_test(test_dstr_regressions ${CMAKE_CURRENT_BINARY_DIR}/test_dstr_regressions)
# A regression of #47 loops forever: fail fast instead.
set_tests_properties(test_dstr_regressions PROPERTIES TIMEOUT 60)

# lexer test
# test_lexer is defined in build/test-graph.json.

# cf lexer test
# test_cf_lexer is defined in build/test-graph.json.

# text lookup test
# test_text_lookup is defined in build/test-graph.json.

# base (logging) test
# test_base is defined in build/test-graph.json.

# bmem test
# test_bmem is defined in build/test-graph.json.

# file serializer test (file-serializer.c, buffered-file-serializer.c)
# test_file_serializer is defined in build/test-graph.json.

# config file test (config-file.c)
# test_config_file is defined in build/test-graph.json.

# threading test (threading-posix.c, threading-windows.c)
add_executable(test_threading test_threading.c)
target_include_directories(test_threading PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_threading PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_link_libraries(test_threading PRIVATE Threads::Threads)
endif()

add_dependencies(cmocka-tests test_threading)
add_test(test_threading ${CMAKE_CURRENT_BINARY_DIR}/test_threading)

# Concurrent reads and writes of the private video-stat storage (#175).
# The sanitizer lane compiles this test with TSan to detect data races.
add_executable(test_video_stats_race test_video_stats_race.c)
target_link_libraries(test_video_stats_race PRIVATE OBS::libobs)
if(NOT MSVC)
  target_link_libraries(test_video_stats_race PRIVATE Threads::Threads)
endif()
add_dependencies(cmocka-tests test_video_stats_race)
add_test(test_video_stats_race ${CMAKE_CURRENT_BINARY_DIR}/test_video_stats_race)

# Qt-free concurrency check for the frontend meter's locked snapshot (#131).
add_executable(test_volume_meter_snapshot test_volume_meter_snapshot.cpp)
if(NOT MSVC)
  target_link_libraries(test_volume_meter_snapshot PRIVATE Threads::Threads)
endif()
add_dependencies(cmocka-tests test_volume_meter_snapshot)
add_test(test_volume_meter_snapshot ${CMAKE_CURRENT_BINARY_DIR}/test_volume_meter_snapshot)

# task test (task.c)
add_executable(test_task test_task.c)
target_include_directories(test_task PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_task PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_link_libraries(test_task PRIVATE Threads::Threads)
endif()

add_dependencies(cmocka-tests test_task)
add_test(test_task ${CMAKE_CURRENT_BINARY_DIR}/test_task)

# pipe test (pipe.c, pipe-posix.c, pipe-windows.c)
# test_pipe is defined in build/test-graph.json.

# profiler test (profiler.c, source-profiler.c)
# test_profiler is defined in build/test-graph.json.

# platform test (platform.c, platform-nix*.c, platform-windows.c)
# test_platform is defined in build/test-graph.json.

# os_dtostr exponent normalization (obs-rust/obs-studio#754)
# test_dtostr_exponent is defined in build/test-graph.json.

# quat test (graphics/quat.h inline setters)
add_executable(test_quat test_quat.c)
target_include_directories(test_quat PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_quat PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_link_libraries(test_quat PRIVATE m)
endif()

add_dependencies(cmocka-tests test_quat)
add_test(test_quat ${CMAKE_CURRENT_BINARY_DIR}/test_quat)

# axisang test (graphics/axisang.c)
add_executable(test_axisang test_axisang.c)
target_include_directories(test_axisang PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_axisang PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_link_libraries(test_axisang PRIVATE m)
endif()

add_dependencies(cmocka-tests test_axisang)
add_test(test_axisang ${CMAKE_CURRENT_BINARY_DIR}/test_axisang)

# vec2 test (graphics/vec2.c)
add_executable(test_vec2 test_vec2.c)
target_include_directories(test_vec2 PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_vec2 PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_link_libraries(test_vec2 PRIVATE m)
endif()

add_dependencies(cmocka-tests test_vec2)
add_test(test_vec2 ${CMAKE_CURRENT_BINARY_DIR}/test_vec2)

# vec3 test (graphics/vec3.c)
add_executable(test_vec3 test_vec3.c)
target_include_directories(test_vec3 PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_vec3 PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_link_libraries(test_vec3 PRIVATE m)
endif()

add_dependencies(cmocka-tests test_vec3)
add_test(test_vec3 ${CMAKE_CURRENT_BINARY_DIR}/test_vec3)

# vec4 test (graphics/vec4.c)
add_executable(test_vec4 test_vec4.c)
target_include_directories(test_vec4 PRIVATE ${CMOCKA_INCLUDE_DIR})
target_link_libraries(test_vec4 PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
if(NOT MSVC)
  target_link_libraries(test_vec4 PRIVATE m)
endif()

add_dependencies(cmocka-tests test_vec4)
add_test(test_vec4 ${CMAKE_CURRENT_BINARY_DIR}/test_vec4)

# video fourcc test (media-io/video-fourcc.c)
# test_video_fourcc is defined in build/test-graph.json.

# video matrices test (media-io/video-matrices.c)
# test_video_matrices is defined in build/test-graph.json.

# video-io.h and frame-rate.h header-inline helpers
# test_video_io_inlines is defined in build/test-graph.json.

# util_uint64.h header-inline helper
# test_util_uint64 is defined in build/test-graph.json.

# Exercise AV1 encoder configuration against real FFmpeg options without a
# running OBS video session, then encode a small synthetic sequence with SVT.
find_package(FFmpeg 8.0 REQUIRED avcodec avformat avutil)
add_executable(
  test_svt_av1
  test_svt_av1.c
  "${CMAKE_SOURCE_DIR}/plugins/obs-ffmpeg/obs-ffmpeg-video-encoders.c.h"
  "${CMAKE_SOURCE_DIR}/shared/opts-parser/opts-parser.c.h"
)
target_compile_definitions(
  test_svt_av1
  PRIVATE
    obs_encoder_video=svt_test_encoder_video
    video_output_get_info=svt_test_video_info
    obs_encoder_get_width=svt_test_encoder_size
    obs_encoder_get_height=svt_test_encoder_size
    obs_encoder_get_name=svt_test_encoder_name
    obs_encoder_set_last_error=svt_test_set_last_error
)
target_include_directories(test_svt_av1 PRIVATE ${CMOCKA_INCLUDE_DIR} "${CMAKE_SOURCE_DIR}/shared/opts-parser")
target_link_libraries(
  test_svt_av1
  PRIVATE OBS::libobs FFmpeg::avcodec FFmpeg::avformat FFmpeg::avutil ${CMOCKA_LIBRARIES}
)
add_dependencies(cmocka-tests test_svt_av1)
add_test(test_svt_av1 ${CMAKE_CURRENT_BINARY_DIR}/test_svt_av1)
set_tests_properties(test_svt_av1 PROPERTIES SKIP_RETURN_CODE 77 TIMEOUT 60)

# Exercise AMF decoding and RTMP server messages through the same
# authentication-enabled sources as obs-outputs, without starting OBS or
# connecting to a server.
if(TARGET OBS::outputs)
  foreach(test_name IN ITEMS test_rtmp_error test_rtmp_chunk_size test_amf_decode)
    add_executable(
      ${test_name}
      ${test_name}.c
      "${CMAKE_SOURCE_DIR}/plugins/obs-outputs/librtmp/rtmp.c"
      "${CMAKE_SOURCE_DIR}/plugins/obs-outputs/librtmp/amf.c"
      "${CMAKE_SOURCE_DIR}/plugins/obs-outputs/librtmp/log.c"
      "${CMAKE_SOURCE_DIR}/plugins/obs-outputs/librtmp/parseurl.c"
      "${CMAKE_SOURCE_DIR}/plugins/obs-outputs/librtmp/hashswf.c"
    )
    target_compile_definitions(${test_name} PRIVATE USE_MBEDTLS CRYPTO)
    target_compile_options(
      ${test_name}
      PRIVATE
        $<$<COMPILE_LANG_AND_ID:C,AppleClang,Clang>:-Wno-comma>
        $<$<AND:$<PLATFORM_ID:Linux,FreeBSD,OpenBSD>,$<COMPILE_LANG_AND_ID:C,Clang>>:-Wno-error=unreachable-code>
    )
    target_include_directories(${test_name} PRIVATE ${CMOCKA_INCLUDE_DIR} "${CMAKE_SOURCE_DIR}/plugins/obs-outputs")
    target_link_libraries(
      ${test_name}
      PRIVATE
        OBS::libobs
        OBS::happy-eyeballs
        MbedTLS::mbedtls
        ZLIB::ZLIB
        ${CMOCKA_LIBRARIES}
        $<$<PLATFORM_ID:Windows>:crypt32>
        $<$<PLATFORM_ID:Windows>:iphlpapi>
        $<$<PLATFORM_ID:Windows>:winmm>
        $<$<PLATFORM_ID:Windows>:ws2_32>
        "$<$<PLATFORM_ID:Darwin>:$<LINK_LIBRARY:FRAMEWORK,Foundation.framework>>"
        "$<$<PLATFORM_ID:Darwin>:$<LINK_LIBRARY:FRAMEWORK,Security.framework>>"
    )
    add_dependencies(cmocka-tests ${test_name})
    add_test(${test_name} ${CMAKE_CURRENT_BINARY_DIR}/${test_name})
  endforeach()

  if(NOT MSVC)
    target_compile_options(test_amf_decode PRIVATE -fsigned-char)
    obs_test_sanitize(test_amf_decode undefined)
  endif()
endif()

# Build obs-outputs' FLV tags from flv-mux.c, without librtmp's network code.
if(TARGET OBS::outputs)
  add_executable(
    test_flv_mux
    test_flv_mux.c
    "${CMAKE_SOURCE_DIR}/plugins/obs-outputs/flv-mux.c.h"
    "${CMAKE_SOURCE_DIR}/plugins/obs-outputs/librtmp/amf.c"
    "${CMAKE_SOURCE_DIR}/plugins/obs-outputs/librtmp/log.c"
  )
  # No network code is built, so librtmp's TLS types are not needed.
  target_compile_definitions(test_flv_mux PRIVATE NO_CRYPTO)
  target_include_directories(test_flv_mux PRIVATE ${CMOCKA_INCLUDE_DIR} "${CMAKE_SOURCE_DIR}/plugins/obs-outputs")
  target_link_libraries(test_flv_mux PRIVATE OBS::libobs ${CMOCKA_LIBRARIES} $<$<PLATFORM_ID:Windows>:ws2_32>)
  add_dependencies(cmocka-tests test_flv_mux)
  add_test(test_flv_mux ${CMAKE_CURRENT_BINARY_DIR}/test_flv_mux)
endif()

# Load .cube LUTs through the Color Grade filter's loader.
if(TARGET OBS::filters)
  add_executable(test_cube_lut test_cube_lut.c "${CMAKE_SOURCE_DIR}/plugins/obs-filters/cube-lut.c.h")
  target_include_directories(test_cube_lut PRIVATE ${CMOCKA_INCLUDE_DIR} "${CMAKE_SOURCE_DIR}/plugins/obs-filters")
  target_link_libraries(test_cube_lut PRIVATE OBS::libobs ${CMOCKA_LIBRARIES} $<$<NOT:$<PLATFORM_ID:Windows>>:m>)
  add_dependencies(cmocka-tests test_cube_lut)
  add_test(test_cube_lut ${CMAKE_CURRENT_BINARY_DIR}/test_cube_lut)
endif()

# Linux capture reconnection after a PipeWire restart (obs-rust/obs-studio#108).
# The decisions are compiled into the tests directly; no PipeWire daemon,
# portal or PulseAudio server is needed.
if(TARGET OBS::pipewire)
  add_executable(
    test_pipewire_reconnect
    test_pipewire_reconnect.c
    "${CMAKE_SOURCE_DIR}/plugins/linux-pipewire/reconnect-policy.c.h"
  )
  target_include_directories(
    test_pipewire_reconnect
    PRIVATE ${CMOCKA_INCLUDE_DIR} "${CMAKE_SOURCE_DIR}/plugins/linux-pipewire"
  )
  target_link_libraries(test_pipewire_reconnect PRIVATE OBS::libobs PipeWire::PipeWire ${CMOCKA_LIBRARIES})
  add_dependencies(cmocka-tests test_pipewire_reconnect)
  add_test(test_pipewire_reconnect ${CMAKE_CURRENT_BINARY_DIR}/test_pipewire_reconnect)
endif()

if(TARGET OBS::pulseaudio)
  add_executable(
    test_pulse_reconnect
    test_pulse_reconnect.c
    "${CMAKE_SOURCE_DIR}/plugins/linux-pulseaudio/pulse-reconnect.c.h"
  )
  target_include_directories(
    test_pulse_reconnect
    PRIVATE ${CMOCKA_INCLUDE_DIR} "${CMAKE_SOURCE_DIR}/plugins/linux-pulseaudio"
  )
  target_link_libraries(test_pulse_reconnect PRIVATE OBS::libobs PulseAudio::PulseAudio ${CMOCKA_LIBRARIES})
  add_dependencies(cmocka-tests test_pulse_reconnect)
  add_test(test_pulse_reconnect ${CMAKE_CURRENT_BINARY_DIR}/test_pulse_reconnect)

  # obs-rust/obs-studio#158: pulseaudio-wrapper.c against a fake libpulse
  # defined in the test (headers only, no libpulse link), with
  # pulseaudio-monitor-format.c, whose device match devices_match() calls.
  add_executable(
    test_pulse_default_id
    test_pulse_default_id.c
    "${CMAKE_SOURCE_DIR}/libobs/audio-monitoring/pulse/pulseaudio-monitor-format.c.h"
    "${CMAKE_SOURCE_DIR}/libobs/audio-monitoring/pulse/pulseaudio-wrapper.c.h"
  )
  target_include_directories(
    test_pulse_default_id
    PRIVATE
      ${CMOCKA_INCLUDE_DIR}
      "${CMAKE_SOURCE_DIR}/libobs/audio-monitoring/pulse"
      $<TARGET_PROPERTY:PulseAudio::PulseAudio,INTERFACE_INCLUDE_DIRECTORIES>
  )
  target_link_libraries(test_pulse_default_id PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
  add_dependencies(cmocka-tests test_pulse_default_id)
  add_test(test_pulse_default_id ${CMAKE_CURRENT_BINARY_DIR}/test_pulse_default_id)
endif()

# video frame test (media-io/video-frame.c + video-frame.h inlines)
# test_video_frame is defined in build/test-graph.json.

# cmocka_qt_runtime(<test> <Qt target>): lets a Qt-linked test find Qt at run
# time outside the app bundle, through the test's environment: macOS gets the
# directory holding the Qt frameworks on DYLD_FRAMEWORK_PATH (a build rpath
# does not reach the binary under the Xcode generator), Windows gets Qt's bin
# directory on PATH and its plugins directory (next to bin, as
# find_qt_plugins() expects) on QT_PLUGIN_PATH. Linux finds Qt through the
# system loader. The path is resolved here: tests added with
# add_test(<name> <cmd>) get no generator expressions in their properties.
function(cmocka_qt_runtime test qt_target)
  get_target_property(qt_location ${qt_target} LOCATION)
  if(NOT qt_location)
    return()
  endif()
  if(OS_MACOS)
    # .../lib/QtCore.framework/Versions/A/QtCore -> .../lib
    string(REGEX REPLACE "/[^/]+\\.framework/.*$" "" qt_dir "${qt_location}")
    set_property(
      TEST ${test}
      APPEND
      PROPERTY ENVIRONMENT_MODIFICATION "DYLD_FRAMEWORK_PATH=path_list_prepend:${qt_dir}"
    )
  elseif(OS_WINDOWS)
    # .../bin/Qt6Core.dll -> .../bin and .../plugins
    get_filename_component(qt_dir "${qt_location}" DIRECTORY)
    set_property(TEST ${test} APPEND PROPERTY ENVIRONMENT_MODIFICATION "PATH=path_list_prepend:${qt_dir}")
    get_filename_component(qt_prefix "${qt_dir}" DIRECTORY)
    if(IS_DIRECTORY "${qt_prefix}/plugins")
      set_property(
        TEST ${test}
        APPEND
        PROPERTY ENVIRONMENT_MODIFICATION "QT_PLUGIN_PATH=path_list_prepend:${qt_prefix}/plugins"
      )
    endif()
  endif()
endfunction()

# One file per test from here on (obs-rust/obs-studio#137): add
# tests.d/<test>.cmake instead of appending to this file, so concurrent PRs
# that add tests do not conflict. Included in this directory's scope, so
# relative source paths and CMAKE_CURRENT_SOURCE_DIR are test/cmocka.
file(GLOB _cmocka_test_fragments CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/tests.d/*.cmake")
list(SORT _cmocka_test_fragments)
foreach(_fragment IN LISTS _cmocka_test_fragments)
  include("${_fragment}")
endforeach()
include("${CMAKE_CURRENT_SOURCE_DIR}/native-test-graph.cmake")
# These explicit false defaults retain the original if() semantics on hosts
# where a platform-only option is not declared.
foreach(_graph_fact IN ITEMS MSVC OS_LINUX ENABLE_PULSEAUDIO ENABLE_HEVC)
  if(NOT DEFINED ${_graph_fact})
    set(${_graph_fact} FALSE)
  endif()
endforeach()
obs_native_test_graph("${CMAKE_CURRENT_SOURCE_DIR}/build/test-graph.json" "${CMAKE_SOURCE_DIR}")
include("${CMAKE_CURRENT_SOURCE_DIR}/parallel-environment.cmake")
