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
    /// 导入文件路径
    import_file_path: String,
    /// 导入状态消息
    import_status: String,
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
            import_status: String::new(),
        }
    }

    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.hotwords.words = self.hotwords.clone();
        config.hotwords.global_weight = self.global_weight;
    }

    /// 从文件导入热词
    fn import_from_file(&mut self, path: &str) -> Result<usize, String> {
        use std::fs;

        let content = fs::read_to_string(path).map_err(|e| format!("无法读取文件: {}", e))?;

        let mut count = 0;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue; // 跳过空行和注释
            }

            // 尝试解析 "词 权重" 格式
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let word = parts[0].to_string();
            let weight = if parts.len() >= 2 {
                parts[1].parse::<f32>().unwrap_or(2.5).clamp(1.0, 5.0)
            } else {
                2.5 // 默认权重
            };

            self.hotwords.insert(word, weight);
            count += 1;
        }

        Ok(count)
    }

    /// 导出热词到文件
    fn export_to_file(&self) -> Result<String, String> {
        use std::fs;
        use std::io::Write;

        // 默认导出路径
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let export_path = format!("{}/vinput-hotwords-export.txt", home);

        let mut file =
            fs::File::create(&export_path).map_err(|e| format!("无法创建文件: {}", e))?;

        // 写入文件头
        writeln!(file, "# V-Input 热词导出文件")
            .map_err(|e| format!("写入失败: {}", e))?;
        writeln!(file, "# 格式: 词 权重")
            .map_err(|e| format!("写入失败: {}", e))?;
        writeln!(file, "# 权重范围: 1.0 - 5.0")
            .map_err(|e| format!("写入失败: {}", e))?;
        writeln!(file, "").map_err(|e| format!("写入失败: {}", e))?;

        // 按字母排序导出
        let mut words: Vec<_> = self.hotwords.iter().collect();
        words.sort_by(|a, b| a.0.cmp(b.0));

        for (word, weight) in words {
            writeln!(file, "{} {:.1}", word, weight).map_err(|e| format!("写入失败: {}", e))?;
        }

        Ok(export_path)
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
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_word)
                        .desired_width(200.0)
                        .hint_text("支持粘贴中文"),
                );
                ui.label("权重:");
                ui.add(egui::Slider::new(&mut self.new_weight, 1.0..=5.0).text(""));
                if ui.button("➕ 添加").clicked() && !self.new_word.is_empty() {
                    self.hotwords.insert(self.new_word.clone(), self.new_weight);
                    self.new_word.clear();
                    self.new_weight = 2.5;
                    modified = true;
                }
            });
            ui.label("💡 提示：可使用 Ctrl+V 粘贴，或使用下方文件导入功能");
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

        // 文件导入功能
        ui.group(|ui| {
            ui.label("📁 从文件导入热词:");
            ui.horizontal(|ui| {
                ui.label("文件路径:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.import_file_path)
                        .desired_width(300.0)
                        .hint_text("/path/to/hotwords.txt"),
                );
                if ui.button("导入").clicked() {
                    let path = self.import_file_path.clone(); // 避免借用冲突
                    match self.import_from_file(&path) {
                        Ok(count) => {
                            self.import_status = format!("✅ 成功导入 {} 个热词", count);
                            modified = true;
                        }
                        Err(e) => {
                            self.import_status = format!("❌ 导入失败: {}", e);
                        }
                    }
                }
            });

            // 显示导入状态
            if !self.import_status.is_empty() {
                ui.label(&self.import_status);
            }

            // 文件格式说明
            ui.collapsing("文件格式说明", |ui| {
                ui.label("支持两种格式:");
                ui.label("1. 简单格式（每行一个词，使用默认权重 2.5）:");
                ui.code("深度操作系统\n语音输入法\n离线识别");
                ui.add_space(5.0);
                ui.label("2. 完整格式（词 + 权重，空格分隔）:");
                ui.code("深度操作系统 3.0\n语音输入法 2.5\n离线识别 3.5");
            });
        });

        ui.add_space(10.0);

        // 导入/导出按钮
        ui.horizontal(|ui| {
            if ui.button("💾 导出到文件").clicked() {
                match self.export_to_file() {
                    Ok(path) => {
                        self.import_status = format!("✅ 已导出到: {}", path);
                    }
                    Err(e) => {
                        self.import_status = format!("❌ 导出失败: {}", e);
                    }
                }
            }
            if ui.button("🗑 清空全部").clicked() {
                self.hotwords.clear();
                modified = true;
                self.import_status = "✅ 已清空所有热词".to_string();
            }
        });

        modified
    }
}
