# The Cargo target triple for the Rust staticlib, so it matches the
# architecture CMake builds libobs for.

# obs_rust_cargo_target(<out> <is_macos> <osx_architectures>)
function(obs_rust_cargo_target out is_macos osx_architectures)
  set(${out} "" PARENT_SCOPE)
endfunction()
