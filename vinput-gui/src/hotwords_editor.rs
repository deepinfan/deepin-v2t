//! 热词编辑器 GUI

use crate::config::{HotwordsConfig, VInputConfig};
use eframe::egui;
use std::collections::HashMap;

pub struct HotwordsEditor {
    /// 热词列表
    hotwords: HashMap<String, f32>,
    /// 全局权重
    global_weight: f32,
    /// 新热词输入
    new_word: String,
    /// 新热词权重
    new_weight: f32,
    /// 要删除的热词
    to_delete: Option<String>,
}

impl HotwordsEditor {
    pub fn new(config: &VInputConfig) -> Self {
        Self {
            hotwords: config.hotwords.words.clone(),
            global_weight: config.hotwords.global_weight,
            new_word: String::new(),
            new_weight: 2.5,
            to_delete: None,
        }
    }

    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.hotwords.words = self.hotwords.clone();
        config.hotwords.global_weight = self.global_weight;
    }

    /// 渲染 UI，返回是否有修改
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.heading("🔥 热词管理");
        ui.separator();

        // 全局设置
        ui.horizontal(|ui| {
            ui.label("全局权重:");
            if ui.add(egui::Slider::new(&mut self.global_weight, 1.0..=5.0)).changed() {
                modified = true;
            }
        });

        ui.add_space(10.0);

        // 添加新热词
        ui.group(|ui| {
            ui.label("添加新热词:");
            ui.horizontal(|ui| {
                ui.label("词汇:");
                ui.text_edit_singleline(&mut self.new_word);
                ui.label("权重:");
                ui.add(egui::Slider::new(&mut self.new_weight, 1.0..=5.0).text(""));
                if ui.button("➕ 添加").clicked() && !self.new_word.is_empty() {
                    self.hotwords.insert(self.new_word.clone(), self.new_weight);
                    self.new_word.clear();
                    self.new_weight = 2.5;
                    modified = true;
                }
            });
        });

        ui.add_space(10.0);

        // 热词列表
        ui.label(format!("热词列表 ({} 个):", self.hotwords.len()));

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                // 简化表格实现
                let mut words: Vec<_> = self.hotwords.iter().map(|(k, v)| (k.clone(), *v)).collect();
                words.sort_by(|a, b| a.0.cmp(&b.0));

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("词汇").strong());
                    ui.add_space(150.0);
                    ui.label(egui::RichText::new("权重").strong());
                    ui.add_space(100.0);
                    ui.label(egui::RichText::new("操作").strong());
                });

                ui.separator();

                let mut updates: Vec<(String, f32)> = Vec::new();

                for (word, weight) in words {
                    ui.horizontal(|ui| {
                        ui.label(&word);
                        ui.add_space(150.0 - word.len() as f32 * 7.0);
                        let mut w = weight;
                        if ui.add(egui::Slider::new(&mut w, 1.0..=5.0).fixed_decimals(1)).changed() {
                            updates.push((word.clone(), w));
                        }
                        ui.add_space(20.0);
                        if ui.button("🗑").clicked() {
                            self.to_delete = Some(word.clone());
                            modified = true;
                        }
                    });
                }

                // 应用权重更新
                for (word, weight) in updates {
                    if let Some(entry) = self.hotwords.get_mut(&word) {
                        *entry = weight;
                        modified = true;
                    }
                }
            });

        // 处理删除
        if let Some(word) = self.to_delete.take() {
            self.hotwords.remove(&word);
        }

        ui.add_space(10.0);

        // 导入/导出按钮
        ui.horizontal(|ui| {
            if ui.button("📁 从文件导入").clicked() {
                // TODO: 文件选择对话框
            }
            if ui.button("💾 导出到文件").clicked() {
                // TODO: 文件保存对话框
            }
            if ui.button("🗑 清空全部").clicked() {
                self.hotwords.clear();
                modified = true;
            }
        });

        modified
    }
}
