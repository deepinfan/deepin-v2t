//! 模型管理面板

use eframe::egui;
use crate::config::VInputConfig;
use std::path::PathBuf;

/// 模型信息
#[derive(Debug, Clone)]
struct ModelInfo {
    name: String,
    path: String,
    size_mb: f64,
    language: String,
    is_installed: bool,
}

/// 模型管理面板
pub struct ModelManagerPanel {
    /// 当前模型目录
    model_dir: String,
    /// 可用模型列表
    available_models: Vec<ModelInfo>,
    /// 状态消息
    status_message: String,
}

impl ModelManagerPanel {
    pub fn new(config: &VInputConfig) -> Self {
        let mut panel = Self {
            model_dir: config.asr.model_dir.clone(),
            available_models: Vec::new(),
            status_message: String::new(),
        };

        panel.scan_models();
        panel
    }

    /// 扫描模型目录
    fn scan_models(&mut self) {
        self.available_models.clear();

        // 预定义的模型列表
        let predefined_models = vec![
            ModelInfo {
                name: "Zipformer 中英双语 (推荐)".to_string(),
                path: "models/zipformer/sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20".to_string(),
                size_mb: 180.0,
                language: "中文+英文".to_string(),
                is_installed: false,
            },
            ModelInfo {
                name: "Zipformer 中文".to_string(),
                path: "models/zipformer/sherpa-onnx-streaming-zipformer-zh-2023-02-20".to_string(),
                size_mb: 150.0,
                language: "中文".to_string(),
                is_installed: false,
            },
            ModelInfo {
                name: "Paraformer 中文".to_string(),
                path: "models/paraformer/sherpa-onnx-paraformer-zh-2023-03-28".to_string(),
                size_mb: 120.0,
                language: "中文".to_string(),
                is_installed: false,
            },
        ];

        // 检查哪些模型已安装
        for mut model in predefined_models {
            let full_path = PathBuf::from(&model.path);
            model.is_installed = full_path.exists();
            self.available_models.push(model);
        }
    }

    /// 渲染 UI
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.heading("模型管理");
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 当前模型
            ui.group(|ui| {
                ui.label("当前使用的模型");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("模型目录:");
                    if ui.text_edit_singleline(&mut self.model_dir).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                if ui.button("📁 选择目录...").clicked() {
                    // TODO: File dialog
                    self.status_message = "文件选择对话框功能待实现".to_string();
                }

                ui.add_space(5.0);
                if ui.button("🔄 刷新模型列表").clicked() {
                    self.scan_models();
                    self.status_message = "已刷新模型列表".to_string();
                }
            });

            ui.add_space(10.0);

            // 可用模型列表
            ui.group(|ui| {
                ui.label("可用模型");
                ui.add_space(5.0);

                for model in &self.available_models {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // 模型名称
                            ui.label(egui::RichText::new(&model.name).strong());

                            ui.add_space(10.0);

                            // 安装状态
                            if model.is_installed {
                                ui.label(egui::RichText::new("✅ 已安装").color(egui::Color32::GREEN));
                            } else {
                                ui.label(egui::RichText::new("❌ 未安装").color(egui::Color32::RED));
                            }
                        });

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            ui.label(format!("语言: {}", model.language));
                            ui.add_space(20.0);
                            ui.label(format!("大小: {:.1} MB", model.size_mb));
                        });

                        ui.add_space(5.0);
                        ui.label(format!("路径: {}", model.path));

                        ui.add_space(5.0);

                        ui.horizontal(|ui| {
                            if model.is_installed {
                                if ui.button("使用此模型").clicked() {
                                    self.model_dir = model.path.clone();
                                    modified = true;
                                    self.status_message = format!("已切换到模型: {}", model.name);
                                }

                                if ui.button("🗑 删除").clicked() {
                                    self.status_message = format!("删除模型功能待实现: {}", model.name);
                                }
                            } else {
                                if ui.button("📥 下载安装").clicked() {
                                    self.status_message = format!("下载功能待实现: {}", model.name);
                                }
                            }
                        });
                    });

                    ui.add_space(5.0);
                }
            });

            ui.add_space(10.0);

            // 模型下载说明
            ui.group(|ui| {
                ui.label("模型下载");
                ui.add_space(5.0);

                ui.label("您可以从以下地址下载模型:");
                ui.hyperlink_to(
                    "sherpa-onnx 模型仓库",
                    "https://github.com/k2-fsa/sherpa-onnx/releases/tag/asr-models"
                );

                ui.add_space(5.0);

                ui.label("下载后解压到以下目录:");
                ui.code("/usr/share/vinput/models/");
                ui.label("或");
                ui.code("~/.local/share/vinput/models/");

                ui.add_space(5.0);

                ui.label("推荐模型:");
                ui.label("• sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20 (中英双语)");
                ui.label("• sherpa-onnx-streaming-zipformer-zh-2023-02-20 (中文)");
            });

            ui.add_space(10.0);

            // 状态消息
            if !self.status_message.is_empty() {
                ui.group(|ui| {
                    ui.label(&self.status_message);
                });
            }
        });

        modified
    }

    /// 应用到配置
    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.asr.model_dir = self.model_dir.clone();
    }
}
