//! VAD/ASR 参数调整面板 GUI

use crate::config::{AsrConfig, VadConfig, VInputConfig};
use eframe::egui;

pub struct VadAsrPanel {
    // VAD 配置
    vad_mode: String,
    start_threshold: f32,
    end_threshold: f32,
    min_speech_duration: u64,
    min_silence_duration: u64,

    // ASR 配置
    model_dir: String,
    sample_rate: i32,
    hotwords_file: String,
    hotwords_score: f32,
}

impl VadAsrPanel {
    pub fn new(config: &VInputConfig) -> Self {
        Self {
            vad_mode: config.vad.mode.clone(),
            start_threshold: config.vad.start_threshold,
            end_threshold: config.vad.end_threshold,
            min_speech_duration: config.vad.min_speech_duration,
            min_silence_duration: config.vad.min_silence_duration,
            model_dir: config.asr.model_dir.clone(),
            sample_rate: config.asr.sample_rate,
            hotwords_file: config.asr.hotwords_file.clone().unwrap_or_default(),
            hotwords_score: config.asr.hotwords_score,
        }
    }

    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.vad.mode = self.vad_mode.clone();
        config.vad.start_threshold = self.start_threshold;
        config.vad.end_threshold = self.end_threshold;
        config.vad.min_speech_duration = self.min_speech_duration;
        config.vad.min_silence_duration = self.min_silence_duration;

        config.asr.model_dir = self.model_dir.clone();
        config.asr.sample_rate = self.sample_rate;
        config.asr.hotwords_file = if self.hotwords_file.is_empty() {
            None
        } else {
            Some(self.hotwords_file.clone())
        };
        config.asr.hotwords_score = self.hotwords_score;
    }

    /// 渲染 UI，返回是否有修改
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.heading("🎤 VAD / ASR 配置");
        ui.separator();

        // VAD 配置
        ui.group(|ui| {
            ui.heading("VAD (语音活动检测)");

            ui.horizontal(|ui| {
                ui.label("VAD 模式:");
                if ui
                    .selectable_label(self.vad_mode == "push-to-talk", "Push-to-Talk")
                    .clicked()
                {
                    self.vad_mode = "push-to-talk".to_string();
                    modified = true;
                }
                if ui
                    .selectable_label(self.vad_mode == "continuous", "Continuous")
                    .clicked()
                {
                    self.vad_mode = "continuous".to_string();
                    modified = true;
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("启动阈值:");
                if ui
                    .add(egui::Slider::new(&mut self.start_threshold, 0.0..=1.0))
                    .changed()
                {
                    modified = true;
                }
            });
            ui.label("检测到语音的概率阈值 (越高越严格)");

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("结束阈值:");
                if ui
                    .add(egui::Slider::new(&mut self.end_threshold, 0.0..=1.0))
                    .changed()
                {
                    modified = true;
                }
            });
            ui.label("检测到静音的概率阈值 (越低越敏感)");

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("最小语音时长:");
                let mut duration_ms = self.min_speech_duration as f32;
                if ui
                    .add(egui::Slider::new(&mut duration_ms, 100.0..=1000.0).suffix(" ms"))
                    .changed()
                {
                    self.min_speech_duration = duration_ms as u64;
                    modified = true;
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("最小静音时长:");
                let mut duration_ms = self.min_silence_duration as f32;
                if ui
                    .add(egui::Slider::new(&mut duration_ms, 100.0..=1000.0).suffix(" ms"))
                    .changed()
                {
                    self.min_silence_duration = duration_ms as u64;
                    modified = true;
                }
            });
        });

        ui.add_space(15.0);

        // ASR 配置
        ui.group(|ui| {
            ui.heading("ASR (语音识别)");

            ui.horizontal(|ui| {
                ui.label("模型目录:");
                if ui.text_edit_singleline(&mut self.model_dir).changed() {
                    modified = true;
                }
                if ui.button("📁").clicked() {
                    // TODO: 文件选择对话框
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("采样率:");
                if ui
                    .selectable_label(self.sample_rate == 16000, "16000 Hz")
                    .clicked()
                {
                    self.sample_rate = 16000;
                    modified = true;
                }
                if ui
                    .selectable_label(self.sample_rate == 8000, "8000 Hz")
                    .clicked()
                {
                    self.sample_rate = 8000;
                    modified = true;
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("热词文件:");
                if ui.text_edit_singleline(&mut self.hotwords_file).changed() {
                    modified = true;
                }
                if ui.button("📁").clicked() {
                    // TODO: 文件选择对话框
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("热词分数:");
                if ui
                    .add(egui::Slider::new(&mut self.hotwords_score, 1.0..=5.0))
                    .changed()
                {
                    modified = true;
                }
            });
            ui.label("热词在识别中的加权分数");
        });

        ui.add_space(15.0);

        // 状态显示
        ui.group(|ui| {
            ui.label("状态:");
            ui.horizontal(|ui| {
                ui.label("● VAD 模式:");
                ui.label(&self.vad_mode);
            });
            ui.horizontal(|ui| {
                ui.label("● ASR 采样率:");
                ui.label(format!("{} Hz", self.sample_rate));
            });
            ui.horizontal(|ui| {
                ui.label("● 模型:");
                if std::path::Path::new(&self.model_dir).exists() {
                    ui.label("✓ 已找到");
                } else {
                    ui.colored_label(egui::Color32::RED, "✗ 未找到");
                }
            });
        });

        modified
    }
}
