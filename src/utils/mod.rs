//! 工具函数模块

use std::time::{Duration, Instant};

/// 动画插值函数

/// easeOutQuad - 二次方缓出
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// easeInQuad - 二次方缓入
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// easeInOutQuad - 二次方缓入缓出
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// easeInOutCubic - 三次方缓入缓出
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// easeOutBounce - 弹跳缓出（用于非法落子回弹效果）
pub fn ease_out_bounce(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

/// 线性插值
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// 动画状态
#[derive(Debug, Clone)]
pub struct Animation {
    pub start_time: Instant,
    pub duration: Duration,
    pub start_value: f32,
    pub end_value: f32,
}

impl Animation {
    pub fn new(duration_ms: u64, start: f32, end: f32) -> Self {
        Self {
            start_time: Instant::now(),
            duration: Duration::from_millis(duration_ms),
            start_value: start,
            end_value: end,
        }
    }

    pub fn progress(&self) -> f32 {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.duration {
            1.0
        } else {
            elapsed.as_secs_f32() / self.duration.as_secs_f32()
        }
    }

    pub fn value(&self) -> f32 {
        let t = ease_out_quad(self.progress());
        lerp(self.start_value, self.end_value, t)
    }

    pub fn is_finished(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }
}

/// 2D向量
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Vec2) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}
