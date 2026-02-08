//! 对话框UI

use rust_i18n::t;
use egui::{Context, Window};

use crate::game::state::GameResult;

/// 新局对话框状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewGameDialog {
    Closed,
    Open,
}

impl NewGameDialog {
    pub fn show(&mut self, ctx: &Context) -> Option<bool> {
        if *self == NewGameDialog::Closed {
            return None;
        }

        let mut result = None;
        let mut open = true;

        Window::new(t!("game.select_side"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(t!("game.select_side"));
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button(format!("🌑 {}", t!("game.play_first"))).clicked() {
                            result = Some(true); // 玩家先行
                            *self = NewGameDialog::Closed;
                        }
                        ui.add_space(20.0);
                        if ui.button(format!("☀️ {}", t!("game.play_second"))).clicked() {
                            result = Some(false); // 玩家后行
                            *self = NewGameDialog::Closed;
                        }
                    });
                });
            });

        if !open {
            *self = NewGameDialog::Closed;
        }

        result
    }
}

/// 游戏结束对话框
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameOverDialog {
    Closed,
    Open(GameResult),
}

impl GameOverDialog {
    pub fn show(&mut self, ctx: &Context) -> Option<GameOverAction> {
        match self {
            GameOverDialog::Closed => return None,
            GameOverDialog::Open(_) => {}
        }

        let mut result = None;
        let mut open = true;
        let result_text = match self {
            GameOverDialog::Open(r) => r.display_text(),
            _ => String::new(),
        };

        Window::new(t!("dialog.game_over"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(&result_text);
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button(format!("🔄 {}", t!("game.undo"))).clicked() {
                            result = Some(GameOverAction::Undo);
                        }
                        ui.add_space(10.0);
                        if ui.button(format!("🎮 {}", t!("game.new_game_btn"))).clicked() {
                            result = Some(GameOverAction::NewGame);
                            *self = GameOverDialog::Closed;
                        }
                        ui.add_space(10.0);
                        if ui.button(format!("🏠 {}", t!("game.back_to_menu"))).clicked() {
                            result = Some(GameOverAction::BackToMenu);
                            *self = GameOverDialog::Closed;
                        }
                    });
                });
            });

        if !open && result.is_none() {
            *self = GameOverDialog::Closed;
        }

        result
    }
}

/// 游戏结束后的操作
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameOverAction {
    Undo,
    NewGame,
    BackToMenu,
}

/// 关于对话框
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AboutDialog {
    Closed,
    Open,
}

impl AboutDialog {
    pub fn show(&mut self, ctx: &Context) {
        if *self == AboutDialog::Closed {
            return;
        }

        let mut open = true;
        Window::new(t!("menu.about"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(t!("app.title"));
                    ui.label("v0.1.0");
                    ui.add_space(10.0);
                    ui.label("一款传统棋类的人机对弈游戏");
                    ui.add_space(10.0);
                    ui.hyperlink_to("项目主页", "https://github.com/yourname/liuzichong");
                });
            });

        if !open {
            *self = AboutDialog::Closed;
        }
    }
}

/// 规则对话框
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RulesDialog {
    Closed,
    Open,
}

impl RulesDialog {
    pub fn show(&mut self, ctx: &Context) {
        if *self == RulesDialog::Closed {
            return;
        }

        let mut open = true;
        Window::new(t!("rules.title"))
            .collapsible(false)
            .resizable(true)
            .default_size([500.0, 400.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(t!("rules.title"));
                    ui.add_space(10.0);
                    
                    ui.label(t!("rules.board"));
                    ui.label(t!("rules.pieces"));
                    ui.label(t!("rules.move"));
                    ui.label(t!("rules.capture"));
                    ui.label(t!("rules.single"));
                    ui.label(t!("rules.draw_rule"));
                    ui.label(t!("rules.stalemate"));
                });
            });

        if !open {
            *self = RulesDialog::Closed;
        }
    }
}
