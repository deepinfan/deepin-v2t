//! 高级设置面板

use eframe::egui;
use crate::config::VInputConfig;

/// 高级设置面板
pub struct AdvancedSettingsPanel {
    /// 日志级别
    log_level: String,
    /// 启用调试模式
    debug_mode: bool,
    /// 性能监控
    performance_monitoring: bool,
    /// 自动保存配置
    auto_save_config: bool,
    /// 配置备份
    config_backup_enabled: bool,
    /// 最大历史记录数
    max_history: usize,
    /// 缓存大小 (MB)
    cache_size_mb: usize,
}

impl AdvancedSettingsPanel {
    pub fn new(_config: &VInputConfig) -> Self {
        Self {
            log_level: "Info".to_string(),
            debug_mode: false,
            performance_monitoring: false,
            auto_save_config: true,
            config_backup_enabled: true,
            max_history: 50,
            cache_size_mb: 100,
        }
    }

    /// 渲染 UI
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.heading("高级设置");
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 日志设置
            ui.group(|ui| {
                ui.label("日志设置");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("日志级别:");
                    egui::ComboBox::from_id_salt("log_level")
                        .selected_text(&self.log_level)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut self.log_level, "Error".to_string(), "Error").clicked() {
                                modified = true;
                            }
                            if ui.selectable_value(&mut self.log_level, "Warn".to_string(), "Warn").clicked() {
                                modified = true;
                            }
                            if ui.selectable_value(&mut self.log_level, "Info".to_string(), "Info").clicked() {
                                modified = true;
                            }
                            if ui.selectable_value(&mut self.log_level, "Debug".to_string(), "Debug").clicked() {
                                modified = true;
                            }
                            if ui.selectable_value(&mut self.log_level, "Trace".to_string(), "Trace").clicked() {
                                modified = true;
                            }
                        });
                });

                ui.add_space(5.0);
                ui.label("提示: Debug 和 Trace 级别会产生大量日志");

                ui.add_space(10.0);

                if ui.checkbox(&mut self.debug_mode, "启用调试模式").changed() {
                    modified = true;
                }
                ui.label("  输出详细的调试信息到日志");

                ui.add_space(10.0);

                if ui.button("📄 查看日志").clicked() {
                    // TODO: Open log viewer
                }

                if ui.button("🗑 清空日志").clicked() {
                    // TODO: Clear logs
                }
            });

            ui.add_space(10.0);

            // 性能设置
            ui.group(|ui| {
                ui.label("性能设置");
                ui.add_space(5.0);

                if ui.checkbox(&mut self.performance_monitoring, "启用性能监控").changed() {
                    modified = true;
                }
                ui.label("  监控 CPU、内存使用情况");

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("缓存大小:");
                    if ui.add(egui::Slider::new(&mut self.cache_size_mb, 50..=500)
                        .suffix(" MB")).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                ui.label("提示: 增大缓存可提升性能，但会占用更多内存");
            });

            ui.add_space(10.0);

            // 配置管理
            ui.group(|ui| {
                ui.label("配置管理");
                ui.add_space(5.0);

                if ui.checkbox(&mut self.auto_save_config, "自动保存配置").changed() {
                    modified = true;
                }
                ui.label("  修改后自动保存，无需手动点击保存按钮");

                ui.add_space(10.0);

                if ui.checkbox(&mut self.config_backup_enabled, "启用配置备份").changed() {
                    modified = true;
                }
                ui.label("  保存配置时自动创建备份");

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("💾 导出配置").clicked() {
                        // TODO: Export config
                    }

                    if ui.button("📥 导入配置").clicked() {
                        // TODO: Import config
                    }

                    if ui.button("🔄 恢复默认").clicked() {
                        // TODO: Reset to defaults
                    }
                });
            });

            ui.add_space(10.0);

            // 历史记录设置
            ui.group(|ui| {
                ui.label("历史记录");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("最大历史记录数:");
                    if ui.add(egui::Slider::new(&mut self.max_history, 10..=200)
                        .suffix(" 条")).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                ui.label("提示: 用于撤销/重试功能");

                ui.add_space(10.0);

                if ui.button("🗑 清空历史记录").clicked() {
                    // TODO: Clear history
                }
            });

            ui.add_space(10.0);

            // 实验性功能
            ui.group(|ui| {
                ui.label("⚠️ 实验性功能");
                ui.add_space(5.0);

                ui.label("以下功能可能不稳定，请谨慎使用:");

                ui.add_space(5.0);

                let mut experimental_feature_1 = false;
                ui.checkbox(&mut experimental_feature_1, "GPU 加速 (实验性)");
                ui.label("  使用 GPU 加速 ASR 推理");

                ui.add_space(5.0);

                let mut experimental_feature_2 = false;
                ui.checkbox(&mut experimental_feature_2, "多线程处理 (实验性)");
                ui.label("  并行处理音频和识别");
            });

            ui.add_space(10.0);

            // 系统信息
            ui.group(|ui| {
                ui.label("系统信息");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("V-Input 版本:");
                    ui.label("0.1.0");
                });

                ui.horizontal(|ui| {
                    ui.label("Rust 版本:");
                    ui.label(env!("CARGO_PKG_RUST_VERSION"));
                });

                ui.horizontal(|ui| {
                    ui.label("操作系统:");
                    ui.label(std::env::consts::OS);
                });

                ui.horizontal(|ui| {
                    ui.label("架构:");
                    ui.label(std::env::consts::ARCH);
                });

                ui.add_space(10.0);

                if ui.button("📋 复制系统信息").clicked() {
                    // TODO: Copy system info to clipboard
                }
            });
        });

        modified
    }

    /// 应用到配置
    pub fn apply_to_config(&self, _config: &mut VInputConfig) {
        // TODO: Add advanced settings to config
    }
}
