//! VAD 模块

pub mod config;
pub mod energy_gate;
pub mod hysteresis;
pub mod pre_roll_buffer;
pub mod transient_filter;
pub mod manager;

#[cfg(feature = "vad-onnx")]
pub mod silero;

pub use config::{EnergyGateConfig, HysteresisConfig, PreRollConfig, SileroConfig, TransientFilterConfig, VadConfig};
pub use energy_gate::EnergyGate;
pub use hysteresis::{HysteresisController, VadState};
pub use manager::{VadManager, VadResult};
pub use pre_roll_buffer::PreRollBuffer;
pub use transient_filter::TransientFilter;

#[cfg(feature = "vad-onnx")]
pub use silero::{SileroVAD, SileroVADConfig, VADState};
