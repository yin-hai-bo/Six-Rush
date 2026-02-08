//! 六子冲 - 人机对弈棋类游戏
//!
//! 游戏规则参见 rules.md
//! 程序规格参见 specification.md

use rust_i18n::i18n;

i18n!("locales", fallback = "zh-CN");

// 导出 t! 宏供外部使用
pub use rust_i18n::t;

pub mod game;
pub mod ui;
pub mod utils;

pub use game::*;
pub use ui::*;

/// 设置当前语言区域
pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}
