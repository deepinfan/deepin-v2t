//! V-Input GUI 设置界面
//!
//! 使用 egui 实现的设置界面，包括：
//! - 热词管理
//! - 标点风格选择
//! - VAD/ASR 参数调整

use eframe::egui;

mod config;
mod hotwords_editor;
mod punctuation_panel;
mod vad_asr_panel;

use config::VInputConfig;
use hotwords_editor::HotwordsEditor;
use punctuation_panel::PunctuationPanel;
use vad_asr_panel::VadAsrPanel;

fn main() -> eframe::Result {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("V-Input 设置"),
        ..Default::default()
    };

    eframe::run_native(
        "V-Input Settings",
        options,
        Box::new(|cc| Ok(Box::new(VInputApp::new(cc)))),
    )
}

/// V-Input 主应用
struct VInputApp {
    /// 当前选项卡
    active_tab: Tab,
    /// 配置
    config: VInputConfig,
    /// 热词编辑器
    hotwords_editor: HotwordsEditor,
    /// 标点面板
    punctuation_panel: PunctuationPanel,
    /// VAD/ASR 面板
    vad_asr_panel: VadAsrPanel,
    /// 配置是否已修改
    config_modified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Hotwords,
    Punctuation,
    VadAsr,
}

impl VInputApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 加载配置
        let config = VInputConfig::load().unwrap_or_default();

        Self {
            active_tab: Tab::Hotwords,
            hotwords_editor: HotwordsEditor::new(&config),
            punctuation_panel: PunctuationPanel::new(&config),
            vad_asr_panel: VadAsrPanel::new(&config),
            config,
            config_modified: false,
        }
    }

    fn save_config(&mut self) {
        // 从各个面板收集配置
        self.hotwords_editor.apply_to_config(&mut self.config);
        self.punctuation_panel.apply_to_config(&mut self.config);
        self.vad_asr_panel.apply_to_config(&mut self.config);

        // 保存到文件
        if let Err(e) = self.config.save() {
            tracing::error!("Failed to save config: {}", e);
        } else {
            self.config_modified = false;
            tracing::info!("Config saved successfully");
        }
    }

    fn reset_config(&mut self) {
        self.config = VInputConfig::default();
        self.hotwords_editor = HotwordsEditor::new(&self.config);
        self.punctuation_panel = PunctuationPanel::new(&self.config);
        self.vad_asr_panel = VadAsrPanel::new(&self.config);
        self.config_modified = true;
    }
}

impl eframe::App for VInputApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 顶部菜单栏
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("保存配置").clicked() {
                        self.save_config();
                        ui.close_menu();
                    }
                    if ui.button("重置为默认").clicked() {
                        self.reset_config();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("退出").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("帮助", |ui| {
                    if ui.button("关于").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        // 底部状态栏
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.config_modified {
                    ui.label("⚠ 配置已修改");
                } else {
                    ui.label("✓ 已保存");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("应用").clicked() {
                        self.save_config();
                    }
                    if ui.button("重置").clicked() {
                        self.reset_config();
                    }
                });
            });
        });

        // 左侧选项卡栏
        egui::SidePanel::left("tab_panel").min_width(120.0).show(ctx, |ui| {
            ui.heading("设置");
            ui.separator();

            if ui
                .selectable_label(self.active_tab == Tab::Hotwords, "🔥 热词管理")
                .clicked()
            {
                self.active_tab = Tab::Hotwords;
            }

            if ui
                .selectable_label(self.active_tab == Tab::Punctuation, "📝 标点控制")
                .clicked()
            {
                self.active_tab = Tab::Punctuation;
            }

            if ui
                .selectable_label(self.active_tab == Tab::VadAsr, "🎤 VAD/ASR")
                .clicked()
            {
                self.active_tab = Tab::VadAsr;
            }
        });

        // 中央面板
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Hotwords => {
                    let modified = self.hotwords_editor.ui(ui);
                    if modified {
                        self.config_modified = true;
                    }
                }
                Tab::Punctuation => {
                    let modified = self.punctuation_panel.ui(ui);
                    if modified {
                        self.config_modified = true;
                    }
                }
                Tab::VadAsr => {
                    let modified = self.vad_asr_panel.ui(ui);
                    if modified {
                        self.config_modified = true;
                    }
                }
            }
        });
    }
}
