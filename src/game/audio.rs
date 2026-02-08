//! 音效系统
//!
//! 按照 specification.md 中的音效规格实现

use rodio::{source::Source, OutputStream, OutputStreamHandle};
use std::collections::HashMap;
use std::io::Cursor;

/// 音效类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundType {
    /// 点击棋子 - 短促点击音，清脆，约100ms
    Click,
    /// 合法落子 - 木质或石质碰撞感
    Place,
    /// 非法落子 - 错误提示音，低沉
    Invalid,
    /// 吃子/担子 - 略长，有"吃掉"的感觉
    Capture,
    /// 玩家获胜 - 胜利音效，欢快
    Win,
    /// 电脑获胜 - 失败音效，低沉
    Lose,
    /// 平局 - 中性音效
    Draw,
}

/// 音效管理器
pub struct AudioManager {
    /// 输出流
    _stream: OutputStream,
    /// 流句柄
    stream_handle: OutputStreamHandle,
    /// 音效缓存
    sounds: HashMap<SoundType, Vec<u8>>,
    /// 是否启用音效
    enabled: bool,
}

impl AudioManager {
    /// 创建新的音效管理器
    pub fn new() -> Option<Self> {
        match OutputStream::try_default() {
            Ok((stream, stream_handle)) => {
                let mut manager = Self {
                    _stream: stream,
                    stream_handle,
                    sounds: HashMap::new(),
                    enabled: true,
                };
                
                // 加载内置音效
                manager.load_builtin_sounds();
                
                Some(manager)
            }
            Err(e) => {
                eprintln!("无法初始化音频系统: {}", e);
                None
            }
        }
    }
    
    /// 加载内置音效（使用程序生成的简单音效）
    fn load_builtin_sounds(&mut self) {
        // 由于嵌入真实音频文件会增加复杂性，
        // 这里使用程序生成的简单音效作为占位符
        // 实际项目中可以使用 include_bytes! 嵌入真实音频文件
        
        // 生成简单的音效数据（正弦波）
        for sound_type in [
            SoundType::Click,
            SoundType::Place,
            SoundType::Invalid,
            SoundType::Capture,
            SoundType::Win,
            SoundType::Lose,
            SoundType::Draw,
        ] {
            let data = self.generate_sound_data(sound_type);
            self.sounds.insert(sound_type, data);
        }
    }
    
    /// 生成简单的音效数据
    fn generate_sound_data(&self, sound_type: SoundType) -> Vec<u8> {
        // 这里生成简单的 WAV 格式音频数据
        // 实际项目中应该加载真实的音频文件
        
        let sample_rate = 44100u32;
        let (frequency, duration_ms, volume) = match sound_type {
            SoundType::Click => (800.0, 100, 0.5),      // 高频短促
            SoundType::Place => (400.0, 200, 0.6),      // 中频中等
            SoundType::Invalid => (200.0, 300, 0.4),    // 低频较长
            SoundType::Capture => (600.0, 400, 0.7),    // 中高频，有层次感
            SoundType::Win => (523.25, 800, 0.8),       // C5音，欢快
            SoundType::Lose => (220.0, 600, 0.5),       // A3音，低沉
            SoundType::Draw => (349.23, 500, 0.5),      // F4音，中性
        };
        
        let num_samples = (sample_rate as f32 * duration_ms as f32 / 1000.0) as usize;
        let mut samples: Vec<i16> = Vec::with_capacity(num_samples);
        
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            // 添加包络使声音更自然
            let envelope = if t < 0.1 {
                t / 0.1 // 攻击阶段
            } else if t > 0.7 {
                (1.0 - t) / 0.3 // 释放阶段
            } else {
                1.0 // 保持阶段
            };
            
            let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin();
            let amplitude = (volume * envelope * 32767.0) as i16;
            samples.push((sample * amplitude as f32) as i16);
        }
        
        // 转换为 WAV 格式
        self.samples_to_wav(&samples, sample_rate)
    }
    
    /// 将样本转换为 WAV 格式
    fn samples_to_wav(&self, samples: &[i16], sample_rate: u32) -> Vec<u8> {
        let num_channels = 1u16;
        let bits_per_sample = 16u16;
        let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
        let block_align = num_channels * (bits_per_sample / 8);
        let data_size = samples.len() as u32 * 2;
        let file_size = 36 + data_size;
        
        let mut wav_data = Vec::with_capacity(file_size as usize);
        
        // RIFF 头
        wav_data.extend_from_slice(b"RIFF");
        wav_data.extend_from_slice(&file_size.to_le_bytes());
        wav_data.extend_from_slice(b"WAVE");
        
        // fmt 块
        wav_data.extend_from_slice(b"fmt ");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // 块大小
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // 音频格式（PCM）
        wav_data.extend_from_slice(&num_channels.to_le_bytes());
        wav_data.extend_from_slice(&sample_rate.to_le_bytes());
        wav_data.extend_from_slice(&byte_rate.to_le_bytes());
        wav_data.extend_from_slice(&block_align.to_le_bytes());
        wav_data.extend_from_slice(&bits_per_sample.to_le_bytes());
        
        // data 块
        wav_data.extend_from_slice(b"data");
        wav_data.extend_from_slice(&data_size.to_le_bytes());
        for sample in samples {
            wav_data.extend_from_slice(&sample.to_le_bytes());
        }
        
        wav_data
    }
    
    /// 播放指定音效
    pub fn play(&self, sound_type: SoundType) {
        if !self.enabled {
            return;
        }
        
        if let Some(data) = self.sounds.get(&sound_type) {
            let cursor = Cursor::new(data.clone());
            if let Ok(source) = rodio::Decoder::new(cursor) {
                let _ = self.stream_handle.play_raw(source.convert_samples());
            }
        }
    }
    
    /// 启用/禁用音效
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// 检查音效是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().expect("无法创建音频管理器")
    }
}

/// 音效播放的简单封装（用于在游戏逻辑中方便调用）
pub struct SoundPlayer {
    audio: Option<AudioManager>,
}

impl SoundPlayer {
    pub fn new() -> Self {
        Self {
            audio: AudioManager::new(),
        }
    }
    
    pub fn play(&self, sound_type: SoundType) {
        if let Some(ref audio) = self.audio {
            audio.play(sound_type);
        }
    }
    
    pub fn click(&self) {
        self.play(SoundType::Click);
    }
    
    pub fn place(&self) {
        self.play(SoundType::Place);
    }
    
    pub fn invalid(&self) {
        self.play(SoundType::Invalid);
    }
    
    pub fn capture(&self) {
        self.play(SoundType::Capture);
    }
    
    pub fn win(&self) {
        self.play(SoundType::Win);
    }
    
    pub fn lose(&self) {
        self.play(SoundType::Lose);
    }
    
    pub fn draw(&self) {
        self.play(SoundType::Draw);
    }
}

impl Default for SoundPlayer {
    fn default() -> Self {
        Self::new()
    }
}
