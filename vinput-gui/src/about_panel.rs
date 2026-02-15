//! 关于页面板

use eframe::egui;
use crate::config::VInputConfig;

/// 关于页面板
pub struct AboutPanel {
    /// 版本信息
    version: String,
    /// 构建日期
    build_date: String,
}

impl AboutPanel {
    pub fn new(_config: &VInputConfig) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_date: "2026-02-15".to_string(),
        }
    }

    /// 渲染 UI
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // Logo 和标题
            ui.heading(egui::RichText::new("V-Input").size(32.0).strong());
            ui.label(egui::RichText::new("离线中文语音输入法").size(18.0));

            ui.add_space(20.0);

            // 版本信息
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.vertical_centered(|ui| {
                    ui.label(format!("版本: {}", self.version));
                    ui.label(format!("构建日期: {}", self.build_date));
                    ui.label("基于 Fcitx5 框架");
                });
            });

            ui.add_space(20.0);

            // 功能特性
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.label(egui::RichText::new("核心特性").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("✅");
                    ui.label("完全离线，保护隐私");
                });

                ui.horizontal(|ui| {
                    ui.label("✅");
                    ui.label("实时流式识别");
                });

                ui.horizontal(|ui| {
                    ui.label("✅");
                    ui.label("智能标点符号");
                });

                ui.horizontal(|ui| {
                    ui.label("✅");
                    ui.label("文本规范化 (ITN)");
                });

                ui.horizontal(|ui| {
                    ui.label("✅");
                    ui.label("热词支持");
                });

                ui.horizontal(|ui| {
                    ui.label("✅");
                    ui.label("撤销/重试功能");
                });
            });

            ui.add_space(20.0);

            // 技术栈
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.label(egui::RichText::new("技术栈").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("🦀");
                    ui.label("Rust - 核心引擎");
                });

                ui.horizontal(|ui| {
                    ui.label("🎤");
                    ui.label("sherpa-onnx - 语音识别");
                });

                ui.horizontal(|ui| {
                    ui.label("🔊");
                    ui.label("PipeWire - 音频捕获");
                });

                ui.horizontal(|ui| {
                    ui.label("⌨️");
                    ui.label("Fcitx5 - 输入法框架");
                });

                ui.horizontal(|ui| {
                    ui.label("🖥️");
                    ui.label("egui - 图形界面");
                });
            });

            ui.add_space(20.0);

            // 链接
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.label(egui::RichText::new("相关链接").strong());
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("📖");
                    ui.hyperlink_to("用户手册", "https://github.com/yourusername/vinput/wiki");
                });

                ui.horizontal(|ui| {
                    ui.label("🐛");
                    ui.hyperlink_to("问题反馈", "https://github.com/yourusername/vinput/issues");
                });

                ui.horizontal(|ui| {
                    ui.label("💻");
                    ui.hyperlink_to("源代码", "https://github.com/yourusername/vinput");
                });

                ui.horizontal(|ui| {
                    ui.label("📄");
                    ui.hyperlink_to("许可证", "https://github.com/yourusername/vinput/blob/main/LICENSE");
                });
            });

            ui.add_space(20.0);

            // 致谢
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.label(egui::RichText::new("致谢").strong());
                ui.add_space(5.0);

                ui.label("感谢以下开源项目:");
                ui.add_space(5.0);

                ui.label("• sherpa-onnx - 语音识别引擎");
                ui.label("• Fcitx5 - 输入法框架");
                ui.label("• PipeWire - 音频服务");
                ui.label("• egui - 即时模式 GUI");
                ui.label("• Rust 社区");
            });

            ui.add_space(20.0);

            // 版权信息
            ui.label(egui::RichText::new("Copyright © 2026 V-Input Contributors").size(12.0));
            ui.label(egui::RichText::new("Licensed under MIT License").size(12.0));

            ui.add_space(20.0);
        });

        false // 关于页不会修改配置
    }

    /// 应用到配置（关于页不需要）
    pub fn apply_to_config(&self, _config: &mut VInputConfig) {
        // 关于页不修改配置
    }
}
