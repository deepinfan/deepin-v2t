cmake_minimum_required(VERSION 3.10)
project(fcitx5-vinput VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 包含标准安装目录定义
include(GNUInstallDirs)

# 查找依赖
find_package(Fcitx5Core REQUIRED)
# Fcitx5Module 不是必需的，移除以避免组件要求错误
find_package(PkgConfig REQUIRED)

# 包含目录
include_directories(
    ${FCITX5_CORE_INCLUDE_DIRS}
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_CURRENT_SOURCE_DIR}/../target
)

# vinput 输入法引擎
add_library(vinput MODULE
    src/vinput_engine.cpp
)

# 链接库
target_link_libraries(vinput
    Fcitx5::Core
    ${CMAKE_CURRENT_SOURCE_DIR}/../target/release/libvinput_core.so
)

# 设置插件属性
set_target_properties(vinput PROPERTIES
    PREFIX ""
    OUTPUT_NAME "vinput"
)

# 安装
install(TARGETS vinput DESTINATION ${CMAKE_INSTALL_LIBDIR}/fcitx5)
install(FILES vinput.conf DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/inputmethod)
install(FILES vinput-addon.conf DESTINATION ${CMAKE_INSTALL_DATADIR}/fcitx5/addon)

# 安装 Rust 核心库
install(FILES ${CMAKE_CURRENT_SOURCE_DIR}/../target/release/libvinput_core.so
        DESTINATION ${CMAKE_INSTALL_LIBDIR})

# 调试信息
message(STATUS "Fcitx5 Core include: ${FCITX5_CORE_INCLUDE_DIRS}")
message(STATUS "Install prefix: ${CMAKE_INSTALL_PREFIX}")
