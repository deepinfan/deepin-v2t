cmake_minimum_required(VERSION 3.10)
project(fcitx5-vinput VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 查找依赖
find_package(Fcitx5Core REQUIRED)
find_package(Fcitx5Module REQUIRED)
find_package(PkgConfig REQUIRED)

# 包含目录
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../vinput-core/target
)

# vinput 输入法引擎
add_library(vinput MODULE
    src/vinput_engine.cpp
    src/vinput_state.cpp
)

# 链接库
target_link_libraries(vinput
    Fcitx5::Core
    Fcitx5::Module
    ${CMAKE_CURRENT_SOURCE_DIR}/../vinput-core/target/release/libvinput_core.so
)

# 安装
install(TARGETS vinput DESTINATION ${CMAKE_INSTALL_LIBDIR}/fcitx5)
install(FILES vinput.conf DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod)
install(FILES vinput-addon.conf DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/addon)
