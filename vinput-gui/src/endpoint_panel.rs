//! 端点检测配置面板

use crate::config::VInputConfig;
use eframe::egui;

/// 标签列固定宽度，与各 group 内所有行一致
const LABEL_W: f32 = 110.0;

pub struct EndpointPanel {
    trailing_silence_ms: u64,
    max_speech_duration_ms: u64,
    vad_start_threshold: f32,
    vad_end_threshold: f32,
    vad_min_speech_duration: u64,
    vad_min_silence_duration: u64,
}

impl EndpointPanel {
    pub fn new(config: &VInputConfig) -> Self {
        Self {
            trailing_silence_ms: config.endpoint.trailing_silence_ms,
            max_speech_duration_ms: config.endpoint.max_speech_duration_ms,
            vad_start_threshold: config.vad.hysteresis.start_threshold,
            vad_end_threshold: config.vad.hysteresis.end_threshold,
            vad_min_speech_duration: config.vad.hysteresis.min_speech_duration_ms,
            vad_min_silence_duration: config.vad.hysteresis.min_silence_duration_ms,
        }
    }

    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.endpoint.trailing_silence_ms = self.trailing_silence_ms;
        config.endpoint.max_speech_duration_ms = self.max_speech_duration_ms;
        config.vad.hysteresis.start_threshold = self.vad_start_threshold;
        config.vad.hysteresis.end_threshold = self.vad_end_threshold;
        config.vad.hysteresis.min_speech_duration_ms = self.vad_min_speech_duration;
        config.vad.hysteresis.min_silence_duration_ms = self.vad_min_silence_duration;
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        // ── 断句延迟 ──────────────────────────────────────────────────────────
        ui.label(egui::RichText::new("断句延迟").size(13.0).strong());
        ui.add_space(4.0);
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("停顿多久后自动上屏（越短越快，越长越稳）")
                .size(12.0).color(egui::Color32::GRAY));
            ui.add_space(4.0);

            // 滑块占满宽度
            let mut v = self.trailing_silence_ms as f32;
            let w = ui.available_width();
            if ui.add_sized([w, 20.0],
                egui::Slider::new(&mut v, 400.0..=2000.0).suffix(" ms"),
            ).changed() {
                self.trailing_silence_ms = v as u64;
                modified = true;
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for (label, val) in [
                    ("快速 600ms", 600u64), ("平衡 800ms ⭐", 800),
                    ("稳定 1000ms", 1000),  ("保守 1500ms", 1500),
                ] {
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

        ui.add_space(8.0);

        // ── 连续说话自动断句 ──────────────────────────────────────────────────────────
        ui.label(egui::RichText::new("连续说话自动断句").size(13.0).strong());
        ui.add_space(4.0);
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("连续说话超过此时长自动断句上屏（0 表示不限制）")
                .size(12.0).color(egui::Color32::GRAY));
            ui.add_space(4.0);

            // 滑块占满宽度
            let mut v = self.max_speech_duration_ms as f32;
            let w = ui.available_width();
            if ui.add_sized([w, 20.0],
                egui::Slider::new(&mut v, 0.0..=60000.0).suffix(" ms"),
            ).changed() {
                self.max_speech_duration_ms = v as u64;
                modified = true;
            }

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for (label, val) in [
                    ("不限制", 0u64), ("10秒", 10000),
                    ("20秒 ⭐", 20000),  ("30秒", 30000),
                ] {
                    if ui.add_sized([90.0, 24.0], egui::SelectableLabel::new(
                        self.max_speech_duration_ms == val,
                        egui::RichText::new(label).size(12.0),
                    )).clicked() {
                        self.max_speech_duration_ms = val;
                        modified = true;
                    }
                    ui.add_space(2.0);
                }
            });
        });

        ui.add_space(8.0);

        // ── 录音参数 ──────────────────────────────────────────────────────────
        ui.label(egui::RichText::new("录音参数").size(13.0).strong());
        ui.add_space(4.0);
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.add_sized([LABEL_W, 16.0],
                    egui::Label::new(egui::RichText::new("麦克风灵敏度").size(13.0)));
                let w = ui.available_width();
                if ui.add_sized([w, 20.0],
                    egui::Slider::new(&mut self.vad_start_threshold, 0.0..=0.4).fixed_decimals(2),
                ).changed() { modified = true; }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_sized([LABEL_W, 16.0],
                    egui::Label::new(egui::RichText::new("静音结束阈值").size(13.0)));
                let w = ui.available_width();
                if ui.add_sized([w, 20.0],
                    egui::Slider::new(&mut self.vad_end_threshold, 0.0..=1.0).fixed_decimals(2),
                ).changed() { modified = true; }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_sized([LABEL_W, 16.0],
                    egui::Label::new(egui::RichText::new("最小语音时长").size(13.0)));
                let mut v = self.vad_min_speech_duration as f32;
                let w = ui.available_width();
                if ui.add_sized([w, 20.0],
                    egui::Slider::new(&mut v, 100.0..=1000.0).suffix(" ms"),
                ).changed() {
                    self.vad_min_speech_duration = v as u64;
                    modified = true;
                }
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.add_sized([LABEL_W, 16.0],
                    egui::Label::new(egui::RichText::new("最小静音时长").size(13.0)));
                let mut v = self.vad_min_silence_duration as f32;
                let w = ui.available_width();
                if ui.add_sized([w, 20.0],
                    egui::Slider::new(&mut v, 100.0..=1000.0).suffix(" ms"),
                ).changed() {
                    self.vad_min_silence_duration = v as u64;
                    modified = true;
                }
            });

            ui.add_space(2.0);
            ui.label(egui::RichText::new("启动阈值越高越严格；结束阈值越低越敏感")
                .size(11.0).color(egui::Color32::GRAY));
        });

        ui.add_space(8.0);

        modified
    }
}
