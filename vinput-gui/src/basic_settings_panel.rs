//! 基本设置面板

use eframe::egui;
use crate::config::VInputConfig;

pub struct BasicSettingsPanel {
    itn_mode: String,
    audio_device: String,
    audio_devices: Vec<(String, String)>,
    language: String,
    /// 当前已保存的热键字符串
    hotkey: String,
    /// 是否处于捕获模式
    capturing: bool,
    /// 已按下但尚未确认的独立修饰键（等待释放来确认是单独按键而非组合键前缀）
    pending_modifier: Option<String>,
    /// 上一帧的修饰键状态（用于检测修饰键按下/释放）
    prev_modifiers: egui::Modifiers,
}

impl BasicSettingsPanel {
    pub fn new(config: &VInputConfig) -> Self {
        Self {
            itn_mode: "Auto".to_string(),
            audio_device: "default".to_string(),
            audio_devices: vec![("default".to_string(), "默认设备".to_string())],
            language: "zh-CN".to_string(),
            hotkey: config.basic.hotkey.clone(),
            capturing: false,
            pending_modifier: None,
            prev_modifiers: egui::Modifiers::NONE,
        }
    }

    pub fn apply_to_config(&self, config: &mut VInputConfig) {
        config.basic.hotkey = self.hotkey.clone();
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        ui.add_space(4.0);
        ui.heading(egui::RichText::new("基本设置").size(18.0).strong());
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // 文本规范化
            ui.label(egui::RichText::new("文本规范化 (ITN)").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                let prev = self.itn_mode.clone();
                ui.radio_value(&mut self.itn_mode, "Auto".to_string(),
                    egui::RichText::new("自动模式  —  启用全部规范化规则（数字、日期、货币等）").size(13.0));
                ui.add_space(2.0);
                ui.radio_value(&mut self.itn_mode, "NumbersOnly".to_string(),
                    egui::RichText::new("仅数字模式  —  仅转换数字，保留其他原始文本").size(13.0));
                ui.add_space(2.0);
                ui.radio_value(&mut self.itn_mode, "Raw".to_string(),
                    egui::RichText::new("原始模式  —  跳过全部规范化，保持识别原文").size(13.0));
                if self.itn_mode != prev { modified = true; }
            });

            ui.add_space(12.0);

            // 识别语言
            ui.label(egui::RichText::new("识别语言").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                let prev = self.language.clone();
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.language, "zh-CN".to_string(), egui::RichText::new("中文").size(13.0));
                    ui.add_space(12.0);
                    ui.radio_value(&mut self.language, "en-US".to_string(), egui::RichText::new("English").size(13.0));
                    ui.add_space(12.0);
                    ui.radio_value(&mut self.language, "zh-en".to_string(), egui::RichText::new("中英混合").size(13.0));
                });
                if self.language != prev { modified = true; }
            });

            ui.add_space(12.0);

            // 音频输入设备
            ui.label(egui::RichText::new("音频输入设备").size(13.0).strong());
            ui.add_space(6.0);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("audio_device")
                        .width(280.0)
                        .selected_text(egui::RichText::new(&self.audio_device).size(13.0))
                        .show_ui(ui, |ui| {
                            for (id, desc) in &self.audio_devices {
                                if ui.selectable_value(&mut self.audio_device, id.clone(),
                                    egui::RichText::new(desc).size(13.0)).clicked() {
                                    modified = true;
                                }
                            }
                        });
                    ui.add_space(8.0);
                    if ui.button(egui::RichText::new("刷新").size(13.0)).clicked() {
                        self.refresh_audio_devices();
                    }
                });
            });

            ui.add_space(12.0);

            // 全局热键
            ui.label(egui::RichText::new("全局热键").size(13.0).strong());
            ui.add_space(6.0);

            if self.capturing {
                modified |= self.ui_capture_mode(ui);
            } else {
                self.ui_display_mode(ui, &mut modified);
            }
        });

        modified
    }

    /// 捕获模式 UI —— 内联显示，返回是否产生修改
    fn ui_capture_mode(&mut self, ui: &mut egui::Ui) -> bool {
        let mut modified = false;

        // ① 先读取本帧所有按键事件（必须在渲染 widget 之前，避免事件被消费）
        #[derive(Debug)]
        enum KeyResult {
            Cancel,
            Captured(String),
        }

        let key_result = ui.input(|i| -> Option<KeyResult> {
            for event in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                    if *key == egui::Key::Escape {
                        return Some(KeyResult::Cancel);
                    }
                    // 任意普通按键（带或不带修饰键均合法）
                    let name = egui_key_name(key);
                    if name == "?" { continue; } // 忽略未知按键
                    let hotkey = if modifiers.any() {
                        format!("{}{}", format_modifiers(modifiers), name)
                    } else {
                        name.to_string()
                    };
                    return Some(KeyResult::Captured(hotkey));
                }
            }
            None
        });

        // ② 读取当前修饰键状态（用于独立修饰键检测）
        let current_mods = ui.input(|i| i.modifiers);

        match key_result {
            Some(KeyResult::Cancel) => {
                // Esc 取消，清空 pending
                self.capturing = false;
                self.pending_modifier = None;
            }
            Some(KeyResult::Captured(hotkey)) => {
                // 普通按键（可能带修饰键）直接确认，忽略 pending
                self.hotkey = hotkey;
                self.capturing = false;
                self.pending_modifier = None;
                modified = true;
            }
            None => {
                // 没有普通按键事件 —— 检测独立修饰键
                if let Some(pending) = self.pending_modifier.take() {
                    // 检查是否有修饰键被释放（按下后松开 = 确认为独立热键）
                    let released =
                        (self.prev_modifiers.ctrl  && !current_mods.ctrl)  ||
                        (self.prev_modifiers.alt   && !current_mods.alt)   ||
                        (self.prev_modifiers.shift && !current_mods.shift);
                    if released {
                        self.hotkey = pending;
                        self.capturing = false;
                        modified = true;
                    } else {
                        // 仍在按住，继续等待
                        self.pending_modifier = Some(pending);
                    }
                } else {
                    // 检测修饰键是否刚被按下（由 false → true）
                    // 注意：此处无法区分左/右 Ctrl，统一记为 RCtrl（与默认热键一致）
                    if !self.prev_modifiers.ctrl && current_mods.ctrl {
                        self.pending_modifier = Some("RCtrl".to_string());
                    } else if !self.prev_modifiers.alt && current_mods.alt {
                        self.pending_modifier = Some("Alt".to_string());
                    } else if !self.prev_modifiers.shift && current_mods.shift {
                        self.pending_modifier = Some("Shift".to_string());
                    }
                }
            }
        }

        // 更新上一帧修饰键状态
        self.prev_modifiers = current_mods;

        // ③ 渲染捕获提示框
        let hint = if let Some(ref pending) = self.pending_modifier {
            format!("已检测到：{}  —  松开按键确认，或继续按其他键组合", hotkey_display(pending))
        } else {
            "请按下目标热键（单键或组合键均可）".to_string()
        };

        let frame_color = if self.pending_modifier.is_some() {
            egui::Color32::from_rgb(30, 80, 50)   // 绿调：已有待确认按键
        } else {
            egui::Color32::from_rgb(35, 55, 100)  // 蓝调：等待输入
        };
        let stroke_color = if self.pending_modifier.is_some() {
            egui::Color32::from_rgb(60, 180, 100)
        } else {
            egui::Color32::from_rgb(80, 130, 220)
        };

        egui::Frame::new()
            .fill(frame_color)
            .stroke(egui::Stroke::new(2.0, stroke_color))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(&hint).size(13.0).color(egui::Color32::WHITE));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("支持：单独的修饰键（Ctrl/Alt/Shift）、功能键（F1–F12）、字母、数字，及任意组合")
                        .size(11.0).color(egui::Color32::from_rgb(160, 185, 220)));
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new("按 Esc 取消").size(11.0)
                        .color(egui::Color32::from_rgb(120, 140, 180)));
                    ui.add_space(4.0);
                });
            });

        // 持续重绘，确保不遗漏事件
        ui.ctx().request_repaint();

        modified
    }

    /// 正常展示模式 UI
    fn ui_display_mode(&mut self, ui: &mut egui::Ui, modified: &mut bool) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("当前热键：").size(13.0));
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(40, 44, 52))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 90, 110)))
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(8, 3))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(hotkey_display(&self.hotkey))
                            .size(13.0).strong()
                            .color(egui::Color32::from_rgb(80, 160, 240)));
                    });
                ui.add_space(12.0);
                if ui.add_sized([60.0, 26.0],
                    egui::Button::new(egui::RichText::new("修改").size(13.0))).clicked() {
                    self.capturing = true;
                    self.pending_modifier = None;
                    self.prev_modifiers = egui::Modifiers::NONE;
                    *modified = false; // 进入捕获模式本身不算修改
                }
            });
            ui.add_space(4.0);
            ui.label(egui::RichText::new("注意：Wayland 下全局热键支持有限").size(11.0)
                .color(egui::Color32::from_rgb(160, 130, 60)));
        });
        let _ = modified; // suppress unused warning
    }

    fn refresh_audio_devices(&mut self) {
        self.audio_devices = vec![
            ("default".to_string(), "默认设备".to_string()),
            ("alsa_input.pci-0000_00_1f.3.analog-stereo".to_string(), "内置音频 模拟立体声".to_string()),
        ];
    }
}

/// 将存储的热键字符串转为友好显示名
fn hotkey_display(hotkey: &str) -> &str {
    match hotkey {
        "RCtrl"  => "右 Ctrl",
        "LCtrl"  => "左 Ctrl",
        "Ctrl"   => "Ctrl",
        "Alt"    => "Alt",
        "Shift"  => "Shift",
        other    => other,
    }
}

/// 将修饰键格式化为前缀字符串（带 + 号后缀）
fn format_modifiers(modifiers: &egui::Modifiers) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if modifiers.ctrl    { parts.push("Ctrl"); }
    if modifiers.alt     { parts.push("Alt"); }
    if modifiers.shift   { parts.push("Shift"); }
    if modifiers.mac_cmd || (modifiers.command && !modifiers.ctrl) {
        parts.push("Super");
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("{}+", parts.join("+"))
    }
}

/// 将 egui Key 转换为可读名称；返回 "?" 表示忽略该键
fn egui_key_name(key: &egui::Key) -> &'static str {
    match key {
        egui::Key::A => "A", egui::Key::B => "B", egui::Key::C => "C",
        egui::Key::D => "D", egui::Key::E => "E", egui::Key::F => "F",
        egui::Key::G => "G", egui::Key::H => "H", egui::Key::I => "I",
        egui::Key::J => "J", egui::Key::K => "K", egui::Key::L => "L",
        egui::Key::M => "M", egui::Key::N => "N", egui::Key::O => "O",
        egui::Key::P => "P", egui::Key::Q => "Q", egui::Key::R => "R",
        egui::Key::S => "S", egui::Key::T => "T", egui::Key::U => "U",
        egui::Key::V => "V", egui::Key::W => "W", egui::Key::X => "X",
        egui::Key::Y => "Y", egui::Key::Z => "Z",
        egui::Key::Num0 => "0", egui::Key::Num1 => "1", egui::Key::Num2 => "2",
        egui::Key::Num3 => "3", egui::Key::Num4 => "4", egui::Key::Num5 => "5",
        egui::Key::Num6 => "6", egui::Key::Num7 => "7", egui::Key::Num8 => "8",
        egui::Key::Num9 => "9",
        egui::Key::F1  => "F1",  egui::Key::F2  => "F2",  egui::Key::F3  => "F3",
        egui::Key::F4  => "F4",  egui::Key::F5  => "F5",  egui::Key::F6  => "F6",
        egui::Key::F7  => "F7",  egui::Key::F8  => "F8",  egui::Key::F9  => "F9",
        egui::Key::F10 => "F10", egui::Key::F11 => "F11", egui::Key::F12 => "F12",
        egui::Key::F13 => "F13", egui::Key::F14 => "F14", egui::Key::F15 => "F15",
        egui::Key::F16 => "F16", egui::Key::F17 => "F17", egui::Key::F18 => "F18",
        egui::Key::F19 => "F19", egui::Key::F20 => "F20",
        egui::Key::Space      => "Space",
        egui::Key::Enter      => "Return",
        egui::Key::Tab        => "Tab",
        egui::Key::Backspace  => "BackSpace",
        egui::Key::Delete     => "Delete",
        egui::Key::Insert     => "Insert",
        egui::Key::Home       => "Home",
        egui::Key::End        => "End",
        egui::Key::PageUp     => "Prior",
        egui::Key::PageDown   => "Next",
        egui::Key::ArrowUp    => "Up",
        egui::Key::ArrowDown  => "Down",
        egui::Key::ArrowLeft  => "Left",
        egui::Key::ArrowRight => "Right",
        _ => "?",
    }
}
