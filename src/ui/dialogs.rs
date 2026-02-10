//! 对话框UI

use rust_i18n::t;
use egui::{Context, Window};

use crate::game::state::GameResult;

/// AI等级选择
pub type AiLevel = u8;

/// 新局对话框结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewGameResult {
    pub player_first: bool,
    pub ai_level: AiLevel,
}

/// 新局对话框状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewGameDialog {
    Closed,
    Open { ai_level: AiLevel },
}

impl Default for NewGameDialog {
    fn default() -> Self {
        NewGameDialog::Open { ai_level: 3 }
    }
}

impl NewGameDialog {
    pub fn show(&mut self, ctx: &Context) -> Option<NewGameResult> {
        match *self {
            NewGameDialog::Closed => return None,
            NewGameDialog::Open { ai_level } => {
                let mut result = None;
                let mut open = true;
                let mut current_level = ai_level;

                Window::new(t!("game.select_side"))
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .open(&mut open)
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            // AI等级选择
                            ui.label(t!("game.ai_level"));
                            ui.add_space(5.0);
                            
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", t!("game.ai_level_label")));
                                ui.add(egui::Slider::new(&mut current_level, 1..=5)
                                    .text("")
                                    .show_value(true));
                            });
                            
                            // 显示当前等级名称
                            let level_name = match current_level {
                                1 => t!("game.ai_level_1"),
                                2 => t!("game.ai_level_2"),
                                3 => t!("game.ai_level_3"),
                                4 => t!("game.ai_level_4"),
                                5 => t!("game.ai_level_5"),
                                _ => t!("game.ai_level_3"),
                            };
                            ui.label(format!("{}: {}", t!("game.ai_level_name"), level_name));
                            ui.add_space(20.0);

                            // 先行/后行选择
                            ui.label(t!("game.select_side_prompt"));
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                if ui.button(format!("🌑 {}", t!("game.play_first"))).clicked() {
                                    result = Some(NewGameResult {
                                        player_first: true,
                                        ai_level: current_level,
                                    });
                                    *self = NewGameDialog::Closed;
                                }
                                ui.add_space(20.0);
                                if ui.button(format!("☀️ {}", t!("game.play_second"))).clicked() {
                                    result = Some(NewGameResult {
                                        player_first: false,
                                        ai_level: current_level,
                                    });
                                    *self = NewGameDialog::Closed;
                                }
                            });
                        });
                    });

                // 更新AI等级状态
                if matches!(*self, NewGameDialog::Open { .. }) {
                    *self = NewGameDialog::Open { ai_level: current_level };
                }

                if !open {
                    *self = NewGameDialog::Closed;
                }

                result
            }
        }
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
