//! 端点检测配置面板

use crate::config::VInputConfig;
use eframe::egui;

pub struct EndpointPanel {
    min_speech_duration_ms: u64,
    max_speech_duration_ms: u64,
    trailing_silence_ms: u64,
    force_timeout_ms: u64,
    vad_assisted: bool,
    vad_silence_confirm_frames: usize,
    vad_start_threshold: f32,
    vad_end_threshold: f32,
    vad_min_speech_duration: u64,
    vad_min_silence_duration: u64,
}

impl EndpointPanel {
    pub fn new(config: &VInputConfig) -> Self {
        Self {
            min_speech_duration_ms: config.endpoint.min_speech_duration_ms,
            max_speech_duration_ms: config.endpoint.max_speech_duration_ms,
            trailing_silence_ms: config.endpoint.trailing_silence_ms,
            force_timeout_ms: config.endpoint.force_timeout_ms,
            vad_assisted: config.endpoint.vad_assisted,
            vad_silence_confirm_frames: config.endpoint.vad_silence_confirm_frames,
            vad_start_threshold: config.vad.start_threshold,
            vad_end_threshold: config.vad.end_threshold,
            vad_min_speech_duration: config.vad.min_speech_duration,
            vad_min_silence_duration: config.vad.min_silence_duration,
        }
    }

    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.endpoint.min_speech_duration_ms = self.min_speech_duration_ms;
        config.endpoint.max_speech_duration_ms = self.max_speech_duration_ms;
        config.endpoint.trailing_silence_ms = self.trailing_silence_ms;
        config.endpoint.force_timeout_ms = self.force_timeout_ms;
        config.endpoint.vad_assisted = self.vad_assisted;
        config.endpoint.vad_silence_confirm_frames = self.vad_silence_confirm_frames;
        config.vad.start_threshold = self.vad_start_threshold;
        config.vad.end_threshold = self.vad_end_threshold;
        config.vad.min_speech_duration = self.vad_min_speech_duration;
        config.vad.min_silence_duration = self.vad_min_silence_duration;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.add_space(4.0);
        ui.heading(egui::RichText::new("端点检测").size(18.0).strong());
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 断句延迟
            ui.label(egui::RichText::new("断句延迟").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.label(egui::RichText::new("停顿多久后自动上屏（越短越快，越长越稳）").size(12.0).color(egui::Color32::GRAY));
                ui.add_space(6.0);

                let mut v = self.trailing_silence_ms as f32;
                if ui.add(egui::Slider::new(&mut v, 400.0..=2000.0).suffix(" ms").text("")).changed() {
                    self.trailing_silence_ms = v as u64;
                    modified = true;
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("快速预设：").size(12.0));
                    for (label, val) in [("快速 600ms", 600u64), ("平衡 800ms ⭐", 800), ("稳定 1000ms", 1000), ("保守 1500ms", 1500)] {
                        if ui.add_sized([90.0, 24.0], egui::SelectableLabel::new(
                            self.trailing_silence_ms == val,
                            egui::RichText::new(label).size(12.0),
                        )).clicked() {
                            self.trailing_silence_ms = val;
                            modified = true;
                        }
                        ui.add_space(2.0);
                    }
                });
            });

            ui.add_space(12.0);

            // 噪声过滤
            ui.label(egui::RichText::new("噪声过滤").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                egui::Grid::new("endpoint_noise_grid")
                    .num_columns(2)
                    .spacing([12.0, 10.0])
                    .min_col_width(120.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("最小语音长度").size(13.0));
                        let mut v = self.min_speech_duration_ms as f32;
                        if ui.add(egui::Slider::new(&mut v, 100.0..=1000.0).suffix(" ms")).changed() {
                            self.min_speech_duration_ms = v as u64;
                            modified = true;
                        }
                        ui.end_row();

                        ui.label(egui::RichText::new("静音确认帧数").size(13.0));
                        let mut v = self.vad_silence_confirm_frames as f32;
                        if ui.add(egui::Slider::new(&mut v, 2.0..=10.0)
                            .suffix(&format!(" 帧 ≈{}ms", self.vad_silence_confirm_frames * 32))).changed() {
                            self.vad_silence_confirm_frames = v as usize;
                            modified = true;
                        }
                        ui.end_row();

                        ui.label(egui::RichText::new("VAD 辅助检测").size(13.0));
                        if ui.checkbox(&mut self.vad_assisted, "").changed() {
                            modified = true;
                        }
                        ui.end_row();
                    });
                ui.label(egui::RichText::new("最小语音长度：短于此时长的音频视为噪声忽略").size(11.0).color(egui::Color32::GRAY));
            });

            ui.add_space(12.0);

            // VAD 参数
            ui.label(egui::RichText::new("VAD 参数").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                egui::Grid::new("vad_grid")
                    .num_columns(2)
                    .spacing([12.0, 10.0])
                    .min_col_width(120.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("语音启动阈值").size(13.0));
                        if ui.add(egui::Slider::new(&mut self.vad_start_threshold, 0.0..=1.0)
                            .fixed_decimals(2)).changed() { modified = true; }
                        ui.end_row();

                        ui.label(egui::RichText::new("静音结束阈值").size(13.0));
                        if ui.add(egui::Slider::new(&mut self.vad_end_threshold, 0.0..=1.0)
                            .fixed_decimals(2)).changed() { modified = true; }
                        ui.end_row();

                        ui.label(egui::RichText::new("最小语音时长").size(13.0));
                        let mut v = self.vad_min_speech_duration as f32;
                        if ui.add(egui::Slider::new(&mut v, 100.0..=1000.0).suffix(" ms")).changed() {
                            self.vad_min_speech_duration = v as u64;
                            modified = true;
                        }
                        ui.end_row();

                        ui.label(egui::RichText::new("最小静音时长").size(13.0));
                        let mut v = self.vad_min_silence_duration as f32;
                        if ui.add(egui::Slider::new(&mut v, 100.0..=1000.0).suffix(" ms")).changed() {
                            self.vad_min_silence_duration = v as u64;
                            modified = true;
                        }
                        ui.end_row();
                    });
                ui.label(egui::RichText::new("启动阈值越高越严格；结束阈值越低越敏感").size(11.0).color(egui::Color32::GRAY));
            });

            ui.add_space(12.0);

            // 高级
            ui.collapsing(egui::RichText::new("高级设置").size(13.0), |ui| {
                ui.add_space(6.0);
                egui::Grid::new("endpoint_adv_grid")
                    .num_columns(2)
                    .spacing([12.0, 10.0])
                    .min_col_width(120.0)
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("最大语音长度").size(13.0));
                        let mut v = (self.max_speech_duration_ms / 1000) as f32;
                        if ui.add(egui::Slider::new(&mut v, 10.0..=180.0).suffix(" 秒")).changed() {
                            self.max_speech_duration_ms = (v * 1000.0) as u64;
                            modified = true;
                        }
                        ui.end_row();

                        ui.label(egui::RichText::new("强制超时").size(13.0));
                        let mut v = (self.force_timeout_ms / 1000) as f32;
                        if ui.add(egui::Slider::new(&mut v, 10.0..=300.0).suffix(" 秒")).changed() {
                            self.force_timeout_ms = (v * 1000.0) as u64;
                            modified = true;
                        }
                        ui.end_row();
                    });
            });
        });

        modified
    }
}
