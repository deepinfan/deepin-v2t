//! 端点检测配置面板

use crate::config::VInputConfig;
use eframe::egui;

/// 端点检测配置面板
pub struct EndpointPanel {
    /// 最小语音长度（毫秒）
    min_speech_duration_ms: u64,
    /// 最大语音长度（毫秒）
    max_speech_duration_ms: u64,
    /// 尾部静音时间（毫秒）
    trailing_silence_ms: u64,
    /// 强制超时（毫秒）
    force_timeout_ms: u64,
    /// VAD 辅助检测
    vad_assisted: bool,
    /// 静音确认帧数
    vad_silence_confirm_frames: usize,
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
        }
    }

    /// 应用配置到 VInputConfig
    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.endpoint.min_speech_duration_ms = self.min_speech_duration_ms;
        config.endpoint.max_speech_duration_ms = self.max_speech_duration_ms;
        config.endpoint.trailing_silence_ms = self.trailing_silence_ms;
        config.endpoint.force_timeout_ms = self.force_timeout_ms;
        config.endpoint.vad_assisted = self.vad_assisted;
        config.endpoint.vad_silence_confirm_frames = self.vad_silence_confirm_frames;
    }

    /// 显示 UI 并返回是否修改
    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.heading("🎯 智能端点检测");
        ui.label("自动识别语音开始和结束，实现智能断句上屏");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // === 基础设置 ===
            ui.group(|ui| {
                ui.heading("📌 基础设置");
                ui.add_space(10.0);

                // VAD 辅助检测
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.vad_assisted, "启用 VAD 辅助检测").changed() {
                        modified = true;
                    }
                    ui.label("✅ 推荐启用，提高检测准确性");
                });

                ui.add_space(5.0);
            });

            ui.add_space(15.0);

            // === 断句延迟设置 ===
            ui.group(|ui| {
                ui.heading("⏱️ 断句延迟（最重要）");
                ui.add_space(10.0);

                ui.label("尾部静音确认时间：");
                ui.label("  停顿多久后自动断句上屏");

                let mut trailing_silence_ms_f = self.trailing_silence_ms as f32;
                ui.add_space(5.0);
                if ui.add(
                    egui::Slider::new(&mut trailing_silence_ms_f, 400.0..=2000.0)
                        .text("毫秒")
                        .suffix(" ms")
                ).changed() {
                    self.trailing_silence_ms = trailing_silence_ms_f as u64;
                    modified = true;
                }

                ui.add_space(10.0);

                // 预设按钮
                ui.horizontal(|ui| {
                    ui.label("快速选择：");
                    if ui.button("快速 (600ms)").clicked() {
                        self.trailing_silence_ms = 600;
                        modified = true;
                    }
                    if ui.button("平衡 (800ms) ⭐").clicked() {
                        self.trailing_silence_ms = 800;
                        modified = true;
                    }
                    if ui.button("稳定 (1000ms)").clicked() {
                        self.trailing_silence_ms = 1000;
                        modified = true;
                    }
                    if ui.button("保守 (1500ms)").clicked() {
                        self.trailing_silence_ms = 1500;
                        modified = true;
                    }
                });

                ui.add_space(10.0);

                // 说明
                ui.label(format!("当前设置: {}ms", self.trailing_silence_ms));
                let desc = match self.trailing_silence_ms {
                    0..=600 => "⚡ 极快响应，但可能误断句",
                    601..=900 => "✅ 平衡模式，推荐使用",
                    901..=1200 => "🛡️ 稳定模式，减少误断",
                    _ => "🐢 保守模式，断句较慢",
                };
                ui.label(desc);
            });

            ui.add_space(15.0);

            // === 噪声过滤 ===
            ui.group(|ui| {
                ui.heading("🔇 噪声过滤");
                ui.add_space(10.0);

                ui.label("最小语音长度：");
                ui.label("  短于此时长的音频会被忽略（过滤点击音、咳嗽等）");

                let mut min_speech_ms_f = self.min_speech_duration_ms as f32;
                ui.add_space(5.0);
                if ui.add(
                    egui::Slider::new(&mut min_speech_ms_f, 100.0..=1000.0)
                        .text("毫秒")
                        .suffix(" ms")
                ).changed() {
                    self.min_speech_duration_ms = min_speech_ms_f as u64;
                    modified = true;
                }

                ui.add_space(10.0);

                ui.label("静音确认帧数：");
                ui.label("  连续 N 帧静音才确认语音结束（1 帧 ≈ 32ms）");

                let mut frames_f = self.vad_silence_confirm_frames as f32;
                ui.add_space(5.0);
                if ui.add(
                    egui::Slider::new(&mut frames_f, 2.0..=10.0)
                        .text("帧")
                        .suffix(&format!(" 帧 (≈{}ms)", self.vad_silence_confirm_frames * 32))
                ).changed() {
                    self.vad_silence_confirm_frames = frames_f as usize;
                    modified = true;
                }
            });

            ui.add_space(15.0);

            // === 高级设置 ===
            ui.collapsing("⚙️ 高级设置", |ui| {
                ui.add_space(10.0);

                // 最大语音长度
                ui.label("最大语音长度（自动分段）：");
                let mut max_speech_sec_f = (self.max_speech_duration_ms / 1000) as f32;
                ui.add_space(5.0);
                if ui.add(
                    egui::Slider::new(&mut max_speech_sec_f, 10.0..=180.0)
                        .text("秒")
                        .suffix(" 秒")
                ).changed() {
                    self.max_speech_duration_ms = (max_speech_sec_f * 1000.0) as u64;
                    modified = true;
                }

                ui.add_space(10.0);

                // 强制超时
                ui.label("强制超时：");
                let mut timeout_sec_f = (self.force_timeout_ms / 1000) as f32;
                ui.add_space(5.0);
                if ui.add(
                    egui::Slider::new(&mut timeout_sec_f, 10.0..=300.0)
                        .text("秒")
                        .suffix(" 秒")
                ).changed() {
                    self.force_timeout_ms = (timeout_sec_f * 1000.0) as u64;
                    modified = true;
                }
            });

            ui.add_space(15.0);

            // === 配置总结 ===
            ui.group(|ui| {
                ui.heading("📊 当前配置总结");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("断句延迟：");
                    ui.label(format!("{}ms", self.trailing_silence_ms));
                });

                ui.horizontal(|ui| {
                    ui.label("最小语音长度：");
                    ui.label(format!("{}ms", self.min_speech_duration_ms));
                });

                ui.horizontal(|ui| {
                    ui.label("静音确认：");
                    ui.label(format!("{} 帧 (≈{}ms)", 
                        self.vad_silence_confirm_frames,
                        self.vad_silence_confirm_frames * 32));
                });

                ui.horizontal(|ui| {
                    ui.label("自动分段：");
                    ui.label(format!("{} 秒", self.max_speech_duration_ms / 1000));
                });

                ui.horizontal(|ui| {
                    ui.label("VAD 辅助：");
                    ui.label(if self.vad_assisted { "✅ 启用" } else { "❌ 禁用" });
                });
            });
        });

        modified
    }
}
