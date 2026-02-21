//! 基本设置面板

use eframe::egui;
use crate::config::VInputConfig;

/// 基本设置面板
pub struct BasicSettingsPanel {
    /// ITN 模式
    itn_mode: String,
    /// 音频设备 ID
    audio_device: String,
    /// 音频设备列表
    audio_devices: Vec<(String, String)>, // (id, description)
    /// 语言
    language: String,
    /// 热键
    hotkey: String,
}

impl BasicSettingsPanel {
    pub fn new(_config: &VInputConfig) -> Self {
        Self {
            itn_mode: "Auto".to_string(), // TODO: Add to config
            audio_device: "default".to_string(), // TODO: Add to config
            audio_devices: vec![("default".to_string(), "默认设备".to_string())],
            language: "zh-CN".to_string(), // TODO: Add to config
            hotkey: "Ctrl+Space".to_string(), // TODO: Add to config
        }
    }

    /// 渲染 UI
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.heading("基本设置");
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ITN 模式
            ui.group(|ui| {
                ui.label("文本规范化 (ITN)");
                ui.add_space(5.0);

                let prev_itn = self.itn_mode.clone();

                ui.radio_value(&mut self.itn_mode, "Auto".to_string(), "自动模式");
                ui.label("  启用全部规范化规则（数字、日期、货币等）");
                ui.add_space(5.0);

                ui.radio_value(&mut self.itn_mode, "NumbersOnly".to_string(), "仅数字模式");
                ui.label("  仅转换数字，保留其他原始文本");
                ui.add_space(5.0);

                ui.radio_value(&mut self.itn_mode, "Raw".to_string(), "原始模式");
                ui.label("  跳过全部规范化，保持识别原文");

                if self.itn_mode != prev_itn {
                    modified = true;
                }
            });

            ui.add_space(10.0);

            // 音频设备
            ui.group(|ui| {
                ui.label("音频输入设备");
                ui.add_space(5.0);

                egui::ComboBox::from_id_salt("audio_device")
                    .selected_text(&self.audio_device)
                    .show_ui(ui, |ui| {
                        for (id, desc) in &self.audio_devices {
                            if ui.selectable_value(&mut self.audio_device, id.clone(), desc).clicked() {
                                modified = true;
                            }
                        }
                    });

                ui.add_space(5.0);
                if ui.button("🔄 刷新设备列表").clicked() {
                    // TODO: Call device enumeration
                    self.refresh_audio_devices();
                }
            });

            ui.add_space(10.0);

            // 语言设置
            ui.group(|ui| {
                ui.label("识别语言");
                ui.add_space(5.0);

                let prev_lang = self.language.clone();

                ui.radio_value(&mut self.language, "zh-CN".to_string(), "中文");
                ui.radio_value(&mut self.language, "en-US".to_string(), "English");
                ui.radio_value(&mut self.language, "zh-en".to_string(), "中英混合");

                if self.language != prev_lang {
                    modified = true;
                }
            });

            ui.add_space(10.0);

            // 热键设置
            ui.group(|ui| {
                ui.label("全局热键");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("当前热键:");
                    ui.label(&self.hotkey);
                });

                ui.add_space(5.0);
                if ui.button("修改热键...").clicked() {
                    // TODO: Implement hotkey capture dialog
                }

                ui.add_space(5.0);
                ui.label("⚠ 注意: Wayland 下热键支持有限");
            });
        });

        modified
    }

    /// 刷新音频设备列表
    fn refresh_audio_devices(&mut self) {
        // TODO: Call FFI to enumerate devices
        // For now, just add a placeholder
        self.audio_devices = vec![
            ("default".to_string(), "默认设备".to_string()),
            ("alsa_input.pci-0000_00_1f.3.analog-stereo".to_string(), "内置音频 模拟立体声".to_string()),
        ];
    }

    /// 应用到配置
    pub fn apply_to_config(&self, _config: &mut VInputConfig) {
        // TODO: Add ITN mode, audio device, language, hotkey to config
    }
}
