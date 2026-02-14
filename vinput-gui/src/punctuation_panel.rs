//! 标点控制面板 GUI

use crate::config::{PunctuationConfig, VInputConfig};
use eframe::egui;

pub struct PunctuationPanel {
    /// 风格
    style: String,
    /// 停顿检测阈值
    pause_ratio: f32,
    /// 最小 token 数
    min_tokens: usize,
    /// 允许感叹号
    allow_exclamation: bool,
    /// 问号严格模式
    question_strict: bool,
}

impl PunctuationPanel {
    pub fn new(config: &VInputConfig) -> Self {
        Self {
            style: config.punctuation.style.clone(),
            pause_ratio: config.punctuation.pause_ratio,
            min_tokens: config.punctuation.min_tokens,
            allow_exclamation: config.punctuation.allow_exclamation,
            question_strict: config.punctuation.question_strict,
        }
    }

    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.punctuation.style = self.style.clone();
        config.punctuation.pause_ratio = self.pause_ratio;
        config.punctuation.min_tokens = self.min_tokens;
        config.punctuation.allow_exclamation = self.allow_exclamation;
        config.punctuation.question_strict = self.question_strict;
    }

    /// 渲染 UI，返回是否有修改
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.heading("📝 标点控制");
        ui.separator();

        // 风格预设
        ui.group(|ui| {
            ui.label("标点风格预设:");
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.style == "Professional", "Professional")
                    .clicked()
                {
                    self.style = "Professional".to_string();
                    self.pause_ratio = 3.5;
                    self.min_tokens = 5;
                    self.allow_exclamation = false;
                    self.question_strict = true;
                    modified = true;
                }

                if ui
                    .selectable_label(self.style == "Balanced", "Balanced")
                    .clicked()
                {
                    self.style = "Balanced".to_string();
                    self.pause_ratio = 2.5;
                    self.min_tokens = 3;
                    self.allow_exclamation = true;
                    self.question_strict = false;
                    modified = true;
                }

                if ui
                    .selectable_label(self.style == "Expressive", "Expressive")
                    .clicked()
                {
                    self.style = "Expressive".to_string();
                    self.pause_ratio = 1.8;
                    self.min_tokens = 2;
                    self.allow_exclamation = true;
                    self.question_strict = false;
                    modified = true;
                }
            });
        });

        ui.add_space(15.0);

        // 停顿检测
        ui.group(|ui| {
            ui.label("停顿检测:");
            ui.horizontal(|ui| {
                ui.label("停顿阈值:");
                if ui
                    .add(egui::Slider::new(&mut self.pause_ratio, 1.0..=5.0).text("x"))
                    .changed()
                {
                    modified = true;
                    self.style = "Custom".to_string();
                }
            });
            ui.label("较大的值 = 需要更长的停顿才插入逗号");

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("最小 token 数:");
                if ui
                    .add(egui::Slider::new(&mut self.min_tokens, 1..=10))
                    .changed()
                {
                    modified = true;
                    self.style = "Custom".to_string();
                }
            });
            ui.label("至少需要多少个词才开始检测停顿");
        });

        ui.add_space(15.0);

        // 标点选项
        ui.group(|ui| {
            ui.label("标点选项:");

            if ui
                .checkbox(&mut self.allow_exclamation, "允许感叹号 (!)")
                .changed()
            {
                modified = true;
                self.style = "Custom".to_string();
            }

            if ui
                .checkbox(&mut self.question_strict, "问号严格模式")
                .changed()
            {
                modified = true;
                self.style = "Custom".to_string();
            }
            ui.label("严格模式：需要声学特征验证");
        });

        ui.add_space(15.0);

        // 预览说明
        ui.group(|ui| {
            ui.label("当前配置说明:");
            match self.style.as_str() {
                "Professional" => {
                    ui.label("✓ 适合正式文档和商务场景");
                    ui.label("✓ 较少的逗号，严格的问号检测");
                    ui.label("✓ 不使用感叹号");
                }
                "Balanced" => {
                    ui.label("✓ 适合日常对话和一般场景");
                    ui.label("✓ 平衡的标点密度");
                    ui.label("✓ 允许感叹号");
                }
                "Expressive" => {
                    ui.label("✓ 适合表达丰富的内容");
                    ui.label("✓ 较多的逗号，宽松的问号检测");
                    ui.label("✓ 允许感叹号");
                }
                _ => {
                    ui.label("✓ 自定义配置");
                }
            }
        });

        modified
    }
}
