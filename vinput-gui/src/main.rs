//! V-Input GUI 设置界面
//!
//! 使用 egui 实现的设置界面，包括：
//! - 热词管理
//! - 标点风格选择
//! - VAD/ASR 参数调整
//! - 端点检测配置

use eframe::egui;

mod config;
mod basic_settings_panel;
mod recognition_settings_panel;
mod advanced_settings_panel;
mod about_panel;
mod endpoint_panel;
mod hotwords_editor;
mod punctuation_panel;
mod vad_asr_panel;

use config::VInputConfig;
use basic_settings_panel::BasicSettingsPanel;
use recognition_settings_panel::RecognitionSettingsPanel;
use advanced_settings_panel::AdvancedSettingsPanel;
use about_panel::AboutPanel;
use endpoint_panel::EndpointPanel;
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
            .with_title("水滴语音输入法 - 设置"),
        ..Default::default()
    };

    eframe::run_native(
        "Droplet Voice Input Settings",
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
    /// 基本设置面板
    basic_settings_panel: BasicSettingsPanel,
    /// 识别设置面板
    recognition_settings_panel: RecognitionSettingsPanel,
    /// 高级设置面板
    advanced_settings_panel: AdvancedSettingsPanel,
    /// 关于面板
    about_panel: AboutPanel,
    /// 热词编辑器
    hotwords_editor: HotwordsEditor,
    /// 标点面板
    punctuation_panel: PunctuationPanel,
    /// VAD/ASR 面板
    vad_asr_panel: VadAsrPanel,
    /// 端点检测面板
    endpoint_panel: EndpointPanel,
    /// 配置是否已修改
    config_modified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Basic,
    Recognition,
    Hotwords,
    Punctuation,
    Advanced,
    VadAsr,
    Endpoint,
    About,
}

impl VInputApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置中文字体支持
        Self::setup_custom_fonts(&cc.egui_ctx);

        // 加载配置
        let config = VInputConfig::load().unwrap_or_default();

        Self {
            active_tab: Tab::Basic,
            basic_settings_panel: BasicSettingsPanel::new(&config),
            recognition_settings_panel: RecognitionSettingsPanel::new(&config),
            advanced_settings_panel: AdvancedSettingsPanel::new(&config),
            about_panel: AboutPanel::new(&config),
            hotwords_editor: HotwordsEditor::new(&config),
            punctuation_panel: PunctuationPanel::new(&config),
            vad_asr_panel: VadAsrPanel::new(&config),
            endpoint_panel: EndpointPanel::new(&config),
            config,
            config_modified: false,
        }
    }

    /// 设置中文字体支持
    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // 尝试加载系统中文字体
        let font_paths = [
            "/usr/share/fonts/opentype/source-han-cjk/SourceHanSansSC-Regular.otf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy-microhei/wqy-microhei.ttc",
        ];

        let mut font_loaded = false;
        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                font_loaded = true;
                tracing::info!("Loaded Chinese font from: {}", font_path);
                break;
            }
        }

        if font_loaded {
            // 将中文字体添加到字体族首位（优先使用）
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese_font".to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese_font".to_owned());
        } else {
            tracing::warn!("No Chinese font found, using default fonts");
        }

        ctx.set_fonts(fonts);
    }

    fn save_config(&mut self) {
        // 从各个面板收集配置
        self.basic_settings_panel.apply_to_config(&mut self.config);
        self.recognition_settings_panel.apply_to_config(&mut self.config);
        self.advanced_settings_panel.apply_to_config(&mut self.config);
        self.hotwords_editor.apply_to_config(&mut self.config);
        self.punctuation_panel.apply_to_config(&mut self.config);
        self.vad_asr_panel.apply_to_config(&mut self.config);
        self.endpoint_panel.apply_to_config(&mut self.config);

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
        self.basic_settings_panel = BasicSettingsPanel::new(&self.config);
        self.recognition_settings_panel = RecognitionSettingsPanel::new(&self.config);
        self.advanced_settings_panel = AdvancedSettingsPanel::new(&self.config);
        self.hotwords_editor = HotwordsEditor::new(&self.config);
        self.punctuation_panel = PunctuationPanel::new(&self.config);
        self.vad_asr_panel = VadAsrPanel::new(&self.config);
        self.endpoint_panel = EndpointPanel::new(&self.config);
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
                .selectable_label(self.active_tab == Tab::Basic, "⚙️ 基本设置")
                .clicked()
            {
                self.active_tab = Tab::Basic;
            }

            if ui
                .selectable_label(self.active_tab == Tab::Recognition, "🎙️ 识别设置")
                .clicked()
            {
                self.active_tab = Tab::Recognition;
            }

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
                .selectable_label(self.active_tab == Tab::Advanced, "🔧 高级设置")
                .clicked()
            {
                self.active_tab = Tab::Advanced;
            }

            if ui
                .selectable_label(self.active_tab == Tab::Endpoint, "🎯 端点检测")
                .clicked()
            {
                self.active_tab = Tab::Endpoint;
            }

            if ui
                .selectable_label(self.active_tab == Tab::VadAsr, "🎤 VAD/ASR")
                .clicked()
            {
                self.active_tab = Tab::VadAsr;
            }

            ui.separator();

            if ui
                .selectable_label(self.active_tab == Tab::About, "ℹ️ 关于")
                .clicked()
            {
                self.active_tab = Tab::About;
            }
        });

        // 中央面板
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Basic => {
                    let modified = self.basic_settings_panel.ui(ui);
                    if modified {
                        self.config_modified = true;
                    }
                }
                Tab::Recognition => {
                    let modified = self.recognition_settings_panel.ui(ui);
                    if modified {
                        self.config_modified = true;
                    }
                }
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
                Tab::Advanced => {
                    let modified = self.advanced_settings_panel.ui(ui);
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
                Tab::Endpoint => {
                    let modified = self.endpoint_panel.ui(ui);
                    if modified {
                        self.config_modified = true;
                    }
                }
                Tab::About => {
                    self.about_panel.ui(ui);
                }
            }
        });
    }
}
