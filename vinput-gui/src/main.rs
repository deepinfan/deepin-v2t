//! V-Input GUI 设置界面

use eframe::egui;
use std::panic;

mod config;
mod basic_settings_panel;
mod about_panel;
mod endpoint_panel;

use config::VInputConfig;
use basic_settings_panel::BasicSettingsPanel;
use about_panel::AboutPanel;
use endpoint_panel::EndpointPanel;

fn main() -> eframe::Result {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([560.0, 540.0])
            .with_min_inner_size([440.0, 360.0])
            .with_resizable(true)
            .with_title("水滴语音输入法 - 设置"),
        ..Default::default()
    };

    eframe::run_native(
        "Droplet Voice Input Settings",
        options,
        Box::new(|cc| Ok(Box::new(VInputApp::new(cc)))),
    )
}

struct VInputApp {
    config: VInputConfig,
    basic_settings_panel: BasicSettingsPanel,
    about_panel: AboutPanel,
    endpoint_panel: EndpointPanel,
    config_modified: bool,
}

impl VInputApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);

        let config = match VInputConfig::load() {
            Ok(cfg) => { tracing::info!("✓ Config loaded"); cfg }
            Err(e) => { tracing::error!("✗ Load failed: {}", e); VInputConfig::default() }
        };

        Self {
            basic_settings_panel: BasicSettingsPanel::new(&config),
            about_panel: AboutPanel::new(&config),
            endpoint_panel: EndpointPanel::new(&config),
            config,
            config_modified: false,
        }
    }

    fn setup_custom_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        let font_paths = [
            "/usr/share/fonts/opentype/source-han-cjk/SourceHanSansSC-Regular.otf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/wqy-microhei/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ];
        for font_path in &font_paths {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_owned(font_data)),
                );
                fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese_font".to_owned());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese_font".to_owned());
                tracing::info!("✓ Font loaded: {}", font_path);
                break;
            }
        }
        ctx.set_fonts(fonts);
    }

    fn save_config(&mut self) {
        self.basic_settings_panel.apply_to_config(&mut self.config);
        self.endpoint_panel.apply_to_config(&mut self.config);
        match self.config.save() {
            Ok(_) => { self.config_modified = false; tracing::info!("Config saved"); }
            Err(e) => tracing::error!("Save failed: {}", e),
        }
    }

    fn reset_config(&mut self) {
        self.config = VInputConfig::default();
        self.basic_settings_panel = BasicSettingsPanel::new(&self.config);
        self.endpoint_panel = EndpointPanel::new(&self.config);
        self.config_modified = true;
    }
}

impl eframe::App for VInputApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 底部状态栏
        egui::TopBottomPanel::bottom("bottom_panel")
            .exact_height(40.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    if self.config_modified {
                        ui.label(egui::RichText::new("● 配置已修改").size(12.0).color(egui::Color32::from_rgb(220, 150, 50)));
                    } else {
                        ui.label(egui::RichText::new("● 已保存").size(12.0).color(egui::Color32::from_rgb(80, 180, 80)));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        if ui.add_sized([60.0, 26.0], egui::Button::new("重置")).clicked() {
                            self.reset_config();
                        }
                        ui.add_space(4.0);
                        if ui.add_sized([60.0, 26.0], egui::Button::new("应用")).clicked() {
                            self.save_config();
                        }
                    });
                });
            });

        // 内容区（单面板顺序显示）
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                // 内边距
                let margin = egui::Margin::symmetric(12, 10);
                egui::Frame::new().inner_margin(margin).show(ui, |ui| {
                    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        if self.basic_settings_panel.ui(ui) { self.config_modified = true; }

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if self.endpoint_panel.ui(ui) { self.config_modified = true; }

                        ui.add_space(10.0);
                        ui.separator();

                        self.about_panel.ui(ui);
                    }));

                    if let Err(e) = result {
                        ui.colored_label(egui::Color32::RED, "⚠ 面板渲染错误");
                        let msg = e.downcast_ref::<String>().cloned()
                            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                            .unwrap_or_else(|| "未知错误".to_string());
                        ui.label(&msg);
                        tracing::error!("Panel panic: {}", msg);
                    }
                });
            });
        });
    }
}
