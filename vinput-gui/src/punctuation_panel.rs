//! 标点控制面板

use crate::config::VInputConfig;
use eframe::egui;

pub struct PunctuationPanel {
    style: String,
    pause_ratio: f32,
    min_tokens: usize,
    allow_exclamation: bool,
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

    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.add_space(4.0);
        ui.heading(egui::RichText::new("标点控制").size(18.0).strong());
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 风格预设
            ui.label(egui::RichText::new("风格预设").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let presets = [
                        ("Professional", "正式", 3.5_f32, 5_usize, false, true),
                        ("Balanced",     "均衡", 2.5,     3,       true,  false),
                        ("Expressive",   "表达", 1.8,     2,       true,  false),
                    ];
                    for (id, label, ratio, tokens, excl, strict) in presets {
                        let active = self.style == id;
                        if ui.add_sized([90.0, 30.0], egui::SelectableLabel::new(active,
                            egui::RichText::new(label).size(13.0))).clicked() && !active {
                            self.style = id.to_string();
                            self.pause_ratio = ratio;
                            self.min_tokens = tokens;
                            self.allow_exclamation = excl;
                            self.question_strict = strict;
                            modified = true;
                        }
                        ui.add_space(4.0);
                    }
                    if self.style == "Custom" {
                        ui.label(egui::RichText::new("自定义").size(12.0)
                            .color(egui::Color32::from_rgb(120, 120, 120)));
                    }
                });
                ui.add_space(4.0);
                let desc = match self.style.as_str() {
                    "Professional" => "适合正式文档：逗号稀少，严格问号，不用感叹号",
                    "Balanced"     => "适合日常对话：标点适中，允许感叹号",
                    "Expressive"   => "适合口语输出：逗号较多，宽松的问号与感叹号",
                    _              => "已手动调整参数",
                };
                ui.label(egui::RichText::new(desc).size(12.0).color(egui::Color32::GRAY));
            });

            ui.add_space(12.0);

            // 详细参数
            ui.label(egui::RichText::new("详细参数").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                egui::Grid::new("punct_grid")
                    .num_columns(2)
                    .spacing([12.0, 10.0])
                    .min_col_width(110.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("停顿阈值").size(13.0));
                        if ui.add(egui::Slider::new(&mut self.pause_ratio, 1.0..=5.0)
                            .suffix("x").fixed_decimals(1)).changed() {
                            modified = true;
                            self.style = "Custom".to_string();
                        }
                        ui.end_row();

                        ui.label(egui::RichText::new("最小词数").size(13.0));
                        if ui.add(egui::Slider::new(&mut self.min_tokens, 1..=10)
                            .suffix(" 词")).changed() {
                            modified = true;
                            self.style = "Custom".to_string();
                        }
                        ui.end_row();
                    });

                ui.add_space(2.0);
                ui.label(egui::RichText::new("停顿阈值越大，需要更长停顿才插入逗号；最小词数越大，短句不插逗号").size(11.0)
                    .color(egui::Color32::GRAY));
            });

            ui.add_space(12.0);

            // 标点开关
            ui.label(egui::RichText::new("标点开关").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                egui::Grid::new("punct_switch_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        if ui.checkbox(&mut self.allow_exclamation,
                            egui::RichText::new("允许感叹号").size(13.0)).changed() {
                            modified = true;
                            self.style = "Custom".to_string();
                        }
                        ui.label(egui::RichText::new("根据语调自动添加 ！").size(12.0).color(egui::Color32::GRAY));
                        ui.end_row();

                        if ui.checkbox(&mut self.question_strict,
                            egui::RichText::new("问号严格模式").size(13.0)).changed() {
                            modified = true;
                            self.style = "Custom".to_string();
                        }
                        ui.label(egui::RichText::new("需要声学特征验证才添加 ？").size(12.0).color(egui::Color32::GRAY));
                        ui.end_row();
                    });
            });
        });

        modified
    }
}
