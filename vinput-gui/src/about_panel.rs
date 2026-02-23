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
            ui.heading(egui::RichText::new("水滴语音输入法").size(32.0).strong());
            ui.label(egui::RichText::new("Droplet Voice Input").size(16.0).color(egui::Color32::GRAY));
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

            // 首发信息
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("首发于深度操作系统论坛").strong());
                    ui.add_space(5.0);
                    ui.hyperlink_to(
                        egui::RichText::new("http://bbs.deepin.org").size(14.0),
                        "http://bbs.deepin.org"
                    );
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

            // 版权信息
            ui.label(egui::RichText::new("Copyright © 2026 水滴语音输入法贡献者").size(12.0));
            ui.label(egui::RichText::new("Licensed under MIT License").size(12.0));

            ui.add_space(20.0);
        });

        false // 关于页不会修改配置
    }

}
