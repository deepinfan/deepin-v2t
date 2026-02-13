cmake_minimum_required(VERSION 3.16)
project(fcitx5-vinput VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 查找 Fcitx5
find_package(Fcitx5Core REQUIRED)
find_package(Fcitx5Module REQUIRED)
find_package(PkgConfig REQUIRED)

# 包含目录
include_directories(
    ${FCITX5_CORE_INCLUDE_DIRS}
    ${CMAKE_CURRENT_SOURCE_DIR}/../target
)

# V-Input 引擎库
set(VINPUT_ENGINE_SOURCES
    src/vinput_engine.cpp
)

# 编译为 Fcitx5 插件
add_library(vinput MODULE ${VINPUT_ENGINE_SOURCES})

# 链接 Fcitx5 和 vinput-core
target_link_libraries(vinput
    Fcitx5::Core
    ${CMAKE_CURRENT_SOURCE_DIR}/../target/release/libvinput_core.so
)

# 设置插件属性
set_target_properties(vinput PROPERTIES
    PREFIX ""
    OUTPUT_NAME "vinput"
)

# 安装目标
install(TARGETS vinput DESTINATION ${CMAKE_INSTALL_LIBDIR}/fcitx5)
install(FILES vinput.conf DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/addon)
install(FILES vinput-im.conf DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod)

# 调试信息
message(STATUS "Fcitx5 Core include: ${FCITX5_CORE_INCLUDE_DIRS}")
message(STATUS "Install prefix: ${CMAKE_INSTALL_PREFIX}")
