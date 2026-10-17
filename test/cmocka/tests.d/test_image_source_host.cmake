# plugins/image-source: the POSIX host's JPEG XR support, file dialog filters
# and stat() sub-second modification time (#858). Compiles
# plugins/image-source/image-source-host-posix.c.h on its own.
if(NOT OS_WINDOWS AND NOT OS_MACOS)
  add_executable(test_image_source_host test_image_source_host.c)
  target_include_directories(test_image_source_host PRIVATE ${CMOCKA_INCLUDE_DIR} "${CMAKE_SOURCE_DIR}")
  target_link_libraries(test_image_source_host PRIVATE OBS::libobs ${CMOCKA_LIBRARIES})
  add_dependencies(cmocka-tests test_image_source_host)
  add_test(test_image_source_host ${CMAKE_CURRENT_BINARY_DIR}/test_image_source_host)
endif()
