//! 热词编辑器

use crate::config::VInputConfig;
use eframe::egui;
use std::collections::HashMap;

pub struct HotwordsEditor {
    hotwords: HashMap<String, f32>,
    global_weight: f32,
    new_word: String,
    new_weight: f32,
    to_delete: Option<String>,
    import_file_path: String,
    status_msg: String,
}

impl HotwordsEditor {
    pub fn new(config: &VInputConfig) -> Self {
        Self {
            hotwords: config.hotwords.words.clone(),
            global_weight: config.hotwords.global_weight,
            new_word: String::new(),
            new_weight: 2.5,
            to_delete: None,
            import_file_path: String::new(),
            status_msg: String::new(),
        }
    }

    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.hotwords.words = self.hotwords.clone();
        config.hotwords.global_weight = self.global_weight;
    }

    fn import_from_file(&mut self, path: &str) -> Result<usize, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("无法读取文件: {}", e))?;
        let mut count = 0;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() { continue; }
            let word = parts[0].to_string();
            let weight = parts.get(1).and_then(|s| s.parse::<f32>().ok()).unwrap_or(2.5).clamp(1.0, 5.0);
            self.hotwords.insert(word, weight);
            count += 1;
        }
        Ok(count)
    }

    fn export_to_file(&self) -> Result<String, String> {
        use std::io::Write;
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let path = format!("{}/vinput-hotwords-export.txt", home);
        let mut file = std::fs::File::create(&path).map_err(|e| format!("无法创建文件: {}", e))?;
        writeln!(file, "# V-Input 热词导出  格式: 词 权重").map_err(|e| format!("{}", e))?;
        let mut words: Vec<_> = self.hotwords.iter().collect();
        words.sort_by(|a, b| a.0.cmp(b.0));
        for (word, weight) in words {
            writeln!(file, "{} {:.1}", word, weight).map_err(|e| format!("{}", e))?;
        }
        Ok(path)
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.add_space(4.0);
        ui.heading(egui::RichText::new("热词管理").size(18.0).strong());
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(8.0);

        // 全局权重
        ui.label(egui::RichText::new("全局权重").size(13.0).strong());
        ui.add_space(6.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("权重倍率：").size(13.0));
                if ui.add(egui::Slider::new(&mut self.global_weight, 1.0..=5.0)
                    .fixed_decimals(1)).changed() { modified = true; }
            });
            ui.label(egui::RichText::new("所有热词的基础权重乘数").size(11.0).color(egui::Color32::GRAY));
        });

        ui.add_space(12.0);

        // 添加热词
        ui.label(egui::RichText::new("添加热词").size(13.0).strong());
        ui.add_space(6.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("词汇：").size(13.0));
                ui.add(egui::TextEdit::singleline(&mut self.new_word)
                    .desired_width(180.0)
                    .font(egui::TextStyle::Body)
                    .hint_text("输入词汇…"));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("权重：").size(13.0));
                ui.add(egui::Slider::new(&mut self.new_weight, 1.0..=5.0)
                    .fixed_decimals(1));
                ui.add_space(8.0);
                let can_add = !self.new_word.is_empty();
                if ui.add_enabled(can_add, egui::Button::new(egui::RichText::new("添加").size(13.0))
                    .min_size([50.0, 0.0].into())).clicked() {
                    self.hotwords.insert(self.new_word.clone(), self.new_weight);
                    self.new_word.clear();
                    self.new_weight = 2.5;
                    modified = true;
                }
            });
        });

        ui.add_space(12.0);

        // 热词列表
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("热词列表（{} 个）", self.hotwords.len())).size(13.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(egui::RichText::new("清空全部").size(12.0)).clicked() {
                    self.hotwords.clear();
                    modified = true;
                    self.status_msg = "已清空所有热词".to_string();
                }
            });
        });
        ui.add_space(6.0);

        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::same(6))
            .show(ui, |ui| {
                // 表头
                egui::Grid::new("hw_header")
                    .num_columns(3)
                    .min_col_width(160.0)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("词汇").size(12.0).strong().color(egui::Color32::GRAY));
                        ui.label(egui::RichText::new("权重").size(12.0).strong().color(egui::Color32::GRAY));
                        ui.label(egui::RichText::new("操作").size(12.0).strong().color(egui::Color32::GRAY));
                        ui.end_row();
                    });
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(180.0)
                    .show(ui, |ui| {
                        let mut words: Vec<_> = self.hotwords.iter().map(|(k, v)| (k.clone(), *v)).collect();
                        words.sort_by(|a, b| a.0.cmp(&b.0));
                        let mut updates: Vec<(String, f32)> = Vec::new();

                        egui::Grid::new("hw_list")
                            .num_columns(3)
                            .min_col_width(160.0)
                            .spacing([8.0, 6.0])
                            .striped(true)
                            .show(ui, |ui| {
                                for (word, weight) in &words {
                                    ui.label(egui::RichText::new(word).size(13.0));
                                    let mut w = *weight;
                                    if ui.add(egui::Slider::new(&mut w, 1.0..=5.0)
                                        .fixed_decimals(1)).changed() {
                                        updates.push((word.clone(), w));
                                    }
                                    if ui.button(egui::RichText::new("删除").size(12.0)).clicked() {
                                        self.to_delete = Some(word.clone());
                                        modified = true;
                                    }
                                    ui.end_row();
                                }
                            });

                        for (word, weight) in updates {
                            if let Some(entry) = self.hotwords.get_mut(&word) {
                                *entry = weight;
                                modified = true;
                            }
                        }
                    });
            });

        if let Some(word) = self.to_delete.take() {
            self.hotwords.remove(&word);
        }

        ui.add_space(12.0);

        // 导入导出
        ui.label(egui::RichText::new("导入 / 导出").size(13.0).strong());
        ui.add_space(6.0);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("文件路径：").size(13.0));
                ui.add(egui::TextEdit::singleline(&mut self.import_file_path)
                    .desired_width(260.0)
                    .font(egui::TextStyle::Body)
                    .hint_text("/path/to/hotwords.txt"));
                ui.add_space(8.0);
                if ui.button(egui::RichText::new("导入").size(13.0)).clicked() {
                    let path = self.import_file_path.clone();
                    match self.import_from_file(&path) {
                        Ok(n)  => { self.status_msg = format!("已导入 {} 个热词", n); modified = true; }
                        Err(e) => { self.status_msg = format!("导入失败：{}", e); }
                    }
                }
                ui.add_space(4.0);
                if ui.button(egui::RichText::new("导出").size(13.0)).clicked() {
                    match self.export_to_file() {
                        Ok(p)  => { self.status_msg = format!("已导出到：{}", p); }
                        Err(e) => { self.status_msg = format!("导出失败：{}", e); }
                    }
                }
            });
            if !self.status_msg.is_empty() {
                ui.add_space(4.0);
                ui.label(egui::RichText::new(&self.status_msg).size(12.0)
                    .color(egui::Color32::from_rgb(80, 160, 80)));
            }
            ui.add_space(2.0);
            ui.label(egui::RichText::new("文件格式：每行一个词，可选权重，如「深度操作系统 3.0」").size(11.0)
                .color(egui::Color32::GRAY));
        });

        modified
    }
}
