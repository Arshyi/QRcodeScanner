# Rust's MSVC targets link the dynamic non-debug CRT in every Cargo profile.
# Force bundled C/C++ dependencies to use the same runtime so `cargo test`
# does not mix /MDd and /MD objects.
set(CMAKE_POLICY_DEFAULT_CMP0091 NEW)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL" CACHE STRING "" FORCE)
