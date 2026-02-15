//! 识别设置面板

use eframe::egui;
use crate::config::VInputConfig;

/// 识别设置面板
pub struct RecognitionSettingsPanel {
    /// 采样率
    sample_rate: i32,
    /// 模型目录
    model_dir: String,
    /// 热词分数
    hotwords_score: f32,
    /// VAD 启动阈值
    vad_start_threshold: f32,
    /// VAD 结束阈值
    vad_end_threshold: f32,
    /// 最小语音时长 (ms)
    min_speech_duration: u64,
    /// 最小静音时长 (ms)
    min_silence_duration: u64,
}

impl RecognitionSettingsPanel {
    pub fn new(config: &VInputConfig) -> Self {
        Self {
            sample_rate: config.asr.sample_rate,
            model_dir: config.asr.model_dir.clone(),
            hotwords_score: config.asr.hotwords_score,
            vad_start_threshold: config.vad.start_threshold,
            vad_end_threshold: config.vad.end_threshold,
            min_speech_duration: config.vad.min_speech_duration,
            min_silence_duration: config.vad.min_silence_duration,
        }
    }

    /// 渲染 UI
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.heading("识别设置");
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // ASR 设置
            ui.group(|ui| {
                ui.label("语音识别 (ASR)");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("模型目录:");
                    if ui.text_edit_singleline(&mut self.model_dir).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                if ui.button("📁 选择模型目录...").clicked() {
                    // TODO: File dialog
                }

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("采样率:");
                    egui::ComboBox::from_id_salt("sample_rate")
                        .selected_text(format!("{} Hz", self.sample_rate))
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut self.sample_rate, 8000, "8000 Hz").clicked() {
                                modified = true;
                            }
                            if ui.selectable_value(&mut self.sample_rate, 16000, "16000 Hz").clicked() {
                                modified = true;
                            }
                            if ui.selectable_value(&mut self.sample_rate, 48000, "48000 Hz").clicked() {
                                modified = true;
                            }
                        });
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("热词权重:");
                    if ui.add(egui::Slider::new(&mut self.hotwords_score, 1.0..=5.0)
                        .step_by(0.1)
                        .text("分数")).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                ui.label("提示: 热词权重越高，热词识别准确率越高，但可能影响其他词汇");
            });

            ui.add_space(10.0);

            // VAD 设置
            ui.group(|ui| {
                ui.label("语音活动检测 (VAD)");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("启动阈值:");
                    if ui.add(egui::Slider::new(&mut self.vad_start_threshold, 0.0..=1.0)
                        .step_by(0.05)
                        .text("概率")).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                ui.label("提示: 阈值越低，越容易触发录音（可能误触发）");

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("结束阈值:");
                    if ui.add(egui::Slider::new(&mut self.vad_end_threshold, 0.0..=1.0)
                        .step_by(0.05)
                        .text("概率")).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                ui.label("提示: 阈值越高，越容易结束录音（可能过早结束）");

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("最小语音时长:");
                    if ui.add(egui::Slider::new(&mut self.min_speech_duration, 100..=1000)
                        .step_by(50.0)
                        .suffix(" ms")).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                ui.label("提示: 过滤掉过短的语音片段");

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("最小静音时长:");
                    if ui.add(egui::Slider::new(&mut self.min_silence_duration, 100..=1000)
                        .step_by(50.0)
                        .suffix(" ms")).changed() {
                        modified = true;
                    }
                });

                ui.add_space(5.0);
                ui.label("提示: 检测到静音后等待多久才判断语音结束");
            });

            ui.add_space(10.0);

            // 预设配置
            ui.group(|ui| {
                ui.label("快速预设");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    if ui.button("🎯 高准确率").clicked() {
                        self.vad_start_threshold = 0.6;
                        self.vad_end_threshold = 0.4;
                        self.min_speech_duration = 300;
                        self.min_silence_duration = 400;
                        self.hotwords_score = 2.0;
                        modified = true;
                    }

                    if ui.button("⚡ 快速响应").clicked() {
                        self.vad_start_threshold = 0.4;
                        self.vad_end_threshold = 0.2;
                        self.min_speech_duration = 200;
                        self.min_silence_duration = 250;
                        self.hotwords_score = 1.5;
                        modified = true;
                    }

                    if ui.button("🔇 抗噪音").clicked() {
                        self.vad_start_threshold = 0.7;
                        self.vad_end_threshold = 0.5;
                        self.min_speech_duration = 400;
                        self.min_silence_duration = 500;
                        self.hotwords_score = 2.5;
                        modified = true;
                    }

                    if ui.button("🔄 默认").clicked() {
                        self.vad_start_threshold = 0.5;
                        self.vad_end_threshold = 0.3;
                        self.min_speech_duration = 250;
                        self.min_silence_duration = 300;
                        self.hotwords_score = 1.5;
                        modified = true;
                    }
                });
            });
        });

        modified
    }

    /// 应用到配置
    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.asr.sample_rate = self.sample_rate;
        config.asr.model_dir = self.model_dir.clone();
        config.asr.hotwords_score = self.hotwords_score;
        config.vad.start_threshold = self.vad_start_threshold;
        config.vad.end_threshold = self.vad_end_threshold;
        config.vad.min_speech_duration = self.min_speech_duration;
        config.vad.min_silence_duration = self.min_silence_duration;
    }
}
