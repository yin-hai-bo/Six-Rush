//! 六子冲游戏程序入口

use eframe::NativeOptions;
use egui::{FontData, FontDefinitions, FontFamily};
use six_rush::ui::MainApp;

// 在二进制 crate 中也初始化 i18n，并导出 t! 宏
rust_i18n::i18n!("locales", fallback = "zh-CN");
pub use rust_i18n::t;

fn main() -> eframe::Result<()> {
    // 设置当前区域为中文
    six_rush::set_locale("zh-CN");

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([900.0, 700.0])
            .with_max_inner_size([900.0, 700.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_decorations(true),
        ..Default::default()
    };

    eframe::run_native(
        &t!("app.title"),
        options,
        Box::new(|cc| Ok(Box::new(setup_app(cc)))),
    )
}

fn setup_app(cc: &eframe::CreationContext<'_>) -> MainApp {
    // 配置中文字体
    setup_fonts(&cc.egui_ctx);

    MainApp::new(cc)
}

/// 配置中文字体支持
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // 尝试加载系统中文字体
    let font_sources = [
        // Windows 中文字体（按优先级排序）
        "C:/Windows/Fonts/msyh.ttc",        // 微软雅黑（优先）
        "C:/Windows/Fonts/msyhbd.ttc",      // 微软雅黑粗体
        "C:/Windows/Fonts/simsun.ttc",      // 宋体（备选）
        "C:/Windows/Fonts/simhei.ttf",      // 黑体（最后备选）
        // Linux 中文字体
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        // macOS 中文字体
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
    ];

    let mut loaded = false;
    for path in font_sources {
        if let Ok(font_data) = std::fs::read(path) {
            // 使用字体文件名（不含扩展名）作为字体名称
            let font_name = std::path::Path::new(path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("CustomFont")
                .to_string();

            fonts.font_data.insert(
                font_name.clone(),
                FontData::from_owned(font_data).into(),
            );

            // 将中文字体添加到 Proportional 和 Monospace 字体族
            if let Some(fonts_for_family) = fonts.families.get_mut(&FontFamily::Proportional) {
                // 将中文字体放在第二位（第一位保留默认字体）
                if fonts_for_family.len() > 1 {
                    fonts_for_family.insert(1, font_name.clone());
                } else {
                    fonts_for_family.push(font_name.clone());
                }
            }

            if let Some(fonts_for_family) = fonts.families.get_mut(&FontFamily::Monospace) {
                fonts_for_family.push(font_name);
            }

            loaded = true;
            break;
        }
    }

    if !loaded {
        // 如果系统字体加载失败，尝试使用 egui 的默认字体配置
        // 或者可以在这里嵌入一个备用字体
        eprintln!("警告：未能加载中文字体，中文可能显示为方块");
    }

    ctx.set_fonts(fonts);
}
