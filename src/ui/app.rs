//! 主应用

use eframe::CreationContext;
use egui::{CentralPanel, Context, Key, TopBottomPanel};
use rust_i18n::t;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::game::audio::SoundPlayer;
use crate::game::board::Board;
use crate::game::piece::Side;
use crate::game::save::{is_initial_position, load_game, save_game};
use crate::game::state::{DialogAction, GameEvent, GameResult, GameState};
use crate::game::Game;
use crate::ui::board_view::BoardView;
use crate::ui::dialogs::{AboutDialog, GameOverAction, GameOverDialog, NewGameDialog, RulesDialog};

/// 动画常量
const PIECE_MOVE_DURATION_MS: u64 = 300;
const PIECE_RETURN_DURATION_MS: u64 = 200;
const CAPTURE_FLASH_DURATION_MS: u64 = 600;
const CAPTURE_REMOVE_DURATION_MS: u64 = 400;
const UNDO_STEP_DURATION_MS: u64 = 400;
const AI_MIN_THINKING_TIME_MS: u64 = 100;

/// 主应用结构
pub struct MainApp {
    /// 游戏状态
    game: Game,
    /// 棋盘视图
    board_view: Option<BoardView>,
    /// 新局对话框
    new_game_dialog: NewGameDialog,
    /// 游戏结束对话框
    game_over_dialog: GameOverDialog,
    /// 关于对话框
    about_dialog: AboutDialog,
    /// 规则对话框
    rules_dialog: RulesDialog,
    /// 动画状态
    animations: AnimationController,
    /// 音效播放器
    sound: SoundPlayer,
    /// 当前语言
    language: String,
    /// 待处理的加载文件路径
    pending_load_file: Option<PathBuf>,
    /// 待处理的保存文件路径
    pending_save_file: Option<PathBuf>,
    /// 确认覆盖对话框状态
    confirm_overwrite: bool,
    /// AI思考开始时间（用于确保最小思考时间）
    ai_think_start: Option<Instant>,
    /// 临时存储的拖拽信息（用于避免借用冲突）
    drag_info: Option<DragInfo>,
}

/// 拖拽信息（从DragState复制，避免借用问题）
#[derive(Debug, Clone, Copy)]
struct DragInfo {
    piece_id: u8,
    start_pos: (u8, u8),
    current_mouse_pos: (f32, f32),
}

/// 动画控制器
#[derive(Debug, Default)]
struct AnimationController {
    /// 棋子移动动画
    piece_move: Option<PieceMoveAnimation>,
    /// 棋子放回原位动画
    piece_return: Option<PieceReturnAnimation>,
    /// 吃子动画
    capture: Option<CaptureAnimation>,
    /// 悔棋动画
    undo: Option<UndoAnimation>,
}

/// 棋子移动动画
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PieceMoveAnimation {
    piece_id: u8,
    from: egui::Pos2,
    to: egui::Pos2,
    start_time: Instant,
    duration_ms: u64,
    is_ai: bool,
}

/// 棋子放回原位动画
#[derive(Debug, Clone)]
struct PieceReturnAnimation {
    piece_id: u8,
    from: egui::Pos2,
    to: egui::Pos2,
    start_time: Instant,
    duration_ms: u64,
}

/// 吃子动画
#[derive(Debug, Clone)]
struct CaptureAnimation {
    piece_ids: Vec<u8>,
    start_time: Instant,
    stage: CaptureStage,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CaptureStage {
    Flashing,
    Removing,
}

/// 悔棋动画
#[allow(dead_code)]
#[derive(Debug)]
struct UndoAnimation {
    step: UndoStep,
    ai_move: PieceMoveAnimation,
    player_move: PieceMoveAnimation,
    ai_record: crate::game::MoveRecord,
    player_record: crate::game::MoveRecord,
    captured_piece: Option<CapturedPieceInfo>,
}

#[derive(Debug, Clone)]
struct CapturedPieceInfo {
    record: crate::game::CapturedRecord,
    screen_pos: egui::Pos2,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum UndoStep {
    AiUndoing,
    CapturedReturning,
    PlayerUndoing,
}

impl MainApp {
    /// 创建新应用
    /// 程序启动时自动开始一局玩家先行的新游戏
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        let mut game = Game::new();
        // 自动开始新局，玩家执黑先行
        let _ = game.handle_event(GameEvent::StartNewGame { player_first: true });

        Self {
            game,
            board_view: None,
            new_game_dialog: NewGameDialog::Closed,
            game_over_dialog: GameOverDialog::Closed,
            about_dialog: AboutDialog::Closed,
            rules_dialog: RulesDialog::Closed,
            animations: AnimationController::default(),
            sound: SoundPlayer::new(),
            language: "zh-CN".to_string(),
            pending_load_file: None,
            pending_save_file: None,
            confirm_overwrite: false,
            ai_think_start: None,
            drag_info: None,
        }
    }

    /// 切换语言
    fn switch_language(&mut self, lang: &str) {
        self.language = lang.to_string();
        rust_i18n::set_locale(lang);
    }

    /// 检查是否有动画正在进行
    fn has_active_animation(&self) -> bool {
        self.animations.piece_move.is_some()
            || self.animations.piece_return.is_some()
            || self.animations.capture.is_some()
            || self.animations.undo.is_some()
    }

    /// 处理菜单栏
    fn handle_menu(&mut self, ctx: &Context) {
        // 只有在可操作UI的状态下才显示/处理菜单
        let can_interact = self.game.state.can_interact_with_ui();
        
        // 处理全局快捷键（当菜单可操作且没有动画时）
        if can_interact && !self.has_active_animation() {
            ctx.input(|i| {
                // F2: 新局, F3: 加载, F4: 保存, Ctrl+Z: 悔棋
                if i.key_pressed(Key::F2) {
                    self.new_game_dialog = NewGameDialog::Open;
                }
                if i.key_pressed(Key::F3) {
                    self.handle_load_game();
                }
                if i.key_pressed(Key::F4) {
                    self.handle_save_game();
                }
                if i.modifiers.ctrl && i.key_pressed(Key::Z) {
                    let _ = self.game.handle_event(GameEvent::StartUndo);
                }
            });
        }

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // 游戏菜单 (ALT+G)
                // 游戏菜单 (支持 ALT+G)
                ui.menu_button(t!("menu.game"), |ui| {
                        let can_click = can_interact && !self.has_active_animation();
                        
                        if ui.add_enabled(can_click, egui::Button::new(t!("menu.new_game"))).clicked() {
                            self.new_game_dialog = NewGameDialog::Open;
                            ui.close_menu();
                        }
                        if ui.add_enabled(can_click, egui::Button::new(t!("menu.load_game"))).clicked() {
                            self.handle_load_game();
                            ui.close_menu();
                        }
                        if ui.add_enabled(can_click, egui::Button::new(t!("menu.save_game"))).clicked() {
                            self.handle_save_game();
                            ui.close_menu();
                        }
                        ui.separator();
                        
                        // 悔棋按钮
                        let can_undo = self.game.can_undo() && can_click;
                        if ui.add_enabled(can_undo, egui::Button::new(t!("menu.undo"))).clicked() {
                            let _ = self.game.handle_event(GameEvent::StartUndo);
                            ui.close_menu();
                        }
                        ui.separator();
                        
                        if ui.button(t!("menu.exit")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            ui.close_menu();
                        }
                    });

                // 语言菜单 (支持 ALT+L)
                ui.menu_button(t!("menu.language"), |ui| {
                        if ui.button(t!("menu.lang_zh")).clicked() {
                            self.switch_language("zh-CN");
                            ui.close_menu();
                        }
                        if ui.button(t!("menu.lang_en")).clicked() {
                            self.switch_language("en");
                            ui.close_menu();
                        }
                });

                // 帮助菜单 (支持 ALT+H)
                ui.menu_button(t!("menu.help"), |ui| {
                        if ui.button(t!("menu.rules")).clicked() {
                            self.rules_dialog = RulesDialog::Open;
                            ui.close_menu();
                        }
                        if ui.button(t!("menu.about")).clicked() {
                            self.about_dialog = AboutDialog::Open;
                            ui.close_menu();
                        }
                    });
            });
        });
    }

    /// 处理快捷工具栏
    fn handle_toolbar(&mut self, ctx: &Context) {
        let can_interact = self.game.state.can_interact_with_ui();

        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let button_size = egui::vec2(72.0, 32.0);
                let can_click = can_interact && !self.has_active_animation();

                // 新局按钮
                let new_game_text = if self.language == "zh-CN" { "🎮 新局" } else { "🎮 New" };
                if ui.add_enabled(can_click, egui::Button::new(new_game_text).min_size(button_size)).clicked() {
                    self.new_game_dialog = NewGameDialog::Open;
                }

                // 保存按钮
                let is_initial = is_initial_position(&self.game.board);
                let can_save = !is_initial && can_click;
                let save_text = if self.language == "zh-CN" { "💾 保存" } else { "💾 Save" };
                if ui.add_enabled(can_save, egui::Button::new(save_text).min_size(button_size)).clicked() {
                    self.handle_save_game();
                }

                // 加载按钮
                let load_text = if self.language == "zh-CN" { "📂 加载" } else { "📂 Load" };
                if ui.add_enabled(can_click, egui::Button::new(load_text).min_size(button_size)).clicked() {
                    self.handle_load_game();
                }

                ui.separator();

                // 悔棋按钮
                let can_undo = self.game.can_undo() && can_click;
                let undo_text = if self.language == "zh-CN" { "↩️ 悔棋" } else { "↩️ Undo" };
                if ui.add_enabled(can_undo, egui::Button::new(undo_text).min_size(button_size)).clicked() {
                    let _ = self.game.handle_event(GameEvent::StartUndo);
                }

                ui.separator();

                // 语言切换按钮
                let lang_text = if self.language == "zh-CN" { "🌐 EN" } else { "🌐 中文" };
                if ui.add_sized(button_size, egui::Button::new(lang_text)).clicked() {
                    if self.language == "zh-CN" {
                        self.switch_language("en");
                    } else {
                        self.switch_language("zh-CN");
                    }
                }

                ui.separator();

                // 规则按钮
                let rules_text = if self.language == "zh-CN" { "📖 规则" } else { "📖 Rules" };
                if ui.add_sized(button_size, egui::Button::new(rules_text)).clicked() {
                    self.rules_dialog = RulesDialog::Open;
                }

                // 关于按钮
                let about_text = if self.language == "zh-CN" { "ℹ️ 关于" } else { "ℹ️ About" };
                if ui.add_sized(button_size, egui::Button::new(about_text)).clicked() {
                    self.about_dialog = AboutDialog::Open;
                }
            });
            ui.add_space(4.0);
        });
    }

    /// 处理新局对话框
    fn handle_new_game_dialog(&mut self, ctx: &Context) {
        if let Some(player_first) = self.new_game_dialog.show(ctx) {
            let _ = self.game.handle_event(GameEvent::StartNewGame { player_first });
            self.animations = AnimationController::default();
            self.ai_think_start = None;
        }
    }

    /// 处理游戏结束对话框
    fn handle_game_over_dialog(&mut self, ctx: &Context) {
        if let Some(action) = self.game_over_dialog.show(ctx) {
            match action {
                GameOverAction::Undo => {
                    let _ = self.game.handle_event(GameEvent::DialogAction(DialogAction::Undo));
                    self.game_over_dialog = GameOverDialog::Closed;
                }
                GameOverAction::NewGame => {
                    self.new_game_dialog = NewGameDialog::Open;
                }
                GameOverAction::BackToMenu => {
                    let _ = self.game.handle_event(GameEvent::DialogAction(DialogAction::Confirm));
                    self.game_over_dialog = GameOverDialog::Closed;
                }
            }
        }
    }

    /// 处理保存游戏
    fn handle_save_game(&mut self) {
        if is_initial_position(&self.game.board) {
            return;
        }

        let dialog = rfd::FileDialog::new()
            .add_filter(&t!("dialog.file_filter"), &["6zc"]);

        if let Some(path) = dialog.save_file() {
            if path.exists() {
                self.pending_save_file = Some(path);
                self.confirm_overwrite = true;
            } else {
                self.do_save_game(&path);
            }
        }
    }

    /// 执行保存游戏
    fn do_save_game(&mut self, path: &std::path::Path) {
        match save_game(&self.game.board, self.game.player_side, path) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("保存游戏失败: {}", e);
            }
        }
        self.pending_save_file = None;
        self.confirm_overwrite = false;
    }

    /// 处理加载游戏
    fn handle_load_game(&mut self) {
        let is_initial = is_initial_position(&self.game.board);

        if !is_initial {
            let dialog = rfd::FileDialog::new()
                .add_filter(&t!("dialog.file_filter"), &["6zc"]);

            if let Some(path) = dialog.pick_file() {
                self.pending_load_file = Some(path);
            }
        } else {
            let dialog = rfd::FileDialog::new()
                .add_filter(&t!("dialog.file_filter"), &["6zc"]);

            if let Some(path) = dialog.pick_file() {
                self.do_load_game(&path);
            }
        }
    }

    /// 执行加载游戏
    fn do_load_game(&mut self, path: &std::path::Path) {
        match load_game(path) {
            Ok((board, player_side)) => {
                self.game.board = board;
                self.game.player_side = player_side;
                self.game.current_turn = Side::Black;
                self.game.state = GameState::WaitingForPlayer;
                self.game.move_history.clear();
                self.game.drag_state = None;
                self.game.pending_move = None;
                self.game.last_captured.clear();
                self.game.last_result = None;
                self.animations = AnimationController::default();
                self.ai_think_start = None;
            }
            Err(e) => {
                eprintln!("加载游戏失败: {}", e);
            }
        }
        self.pending_load_file = None;
    }

    /// 显示确认加载对话框
    fn show_confirm_load_dialog(&mut self, ctx: &Context) {
        if let Some(ref path) = self.pending_load_file.clone() {
            let mut should_load = false;
            let mut should_cancel = false;

            egui::Window::new(t!("dialog.confirm_load"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(t!("dialog.confirm_load_msg"));
                    ui.horizontal(|ui| {
                        if ui.button(t!("dialog.yes")).clicked() {
                            should_load = true;
                        }
                        if ui.button(t!("dialog.no")).clicked() {
                            should_cancel = true;
                        }
                    });
                });

            if should_load {
                self.do_load_game(path);
            } else if should_cancel {
                self.pending_load_file = None;
            }
        }
    }

    /// 显示确认覆盖对话框
    fn show_confirm_overwrite_dialog(&mut self, ctx: &Context) {
        if let Some(ref path) = self.pending_save_file.clone() {
            let mut should_save = false;
            let mut should_cancel = false;

            egui::Window::new(t!("dialog.confirm_overwrite"))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(t!("dialog.confirm_overwrite_msg"));
                    ui.horizontal(|ui| {
                        if ui.button(t!("dialog.yes")).clicked() {
                            should_save = true;
                        }
                        if ui.button(t!("dialog.no")).clicked() {
                            should_cancel = true;
                        }
                    });
                });

            if should_save {
                self.do_save_game(path);
            } else if should_cancel {
                self.pending_save_file = None;
                self.confirm_overwrite = false;
            }
        }
    }

    /// 处理AI回合
    fn handle_ai_turn(&mut self) {
        // 确保有动画正在进行时等待
        if self.has_active_animation() {
            return;
        }

        // 记录AI思考开始时间
        if self.ai_think_start.is_none() {
            self.ai_think_start = Some(Instant::now());
        }

        let elapsed = self.ai_think_start.unwrap().elapsed();
        
        // 确保最小思考时间（100ms）
        if elapsed < Duration::from_millis(AI_MIN_THINKING_TIME_MS) {
            return;
        }

        // 执行AI移动
        use crate::game::ai::AiPlayer;
        let ai = AiPlayer::new(self.game.ai_level);
        
        match ai.select_move(&self.game.board, self.game.player_side.opposite()) {
            Ok((from, to)) => {
                let _ = self.game.handle_event(GameEvent::AiMoveSelected { from, to });
                
                // 触发移动动画
                if let Some(ref view) = self.board_view {
                    let from_pos = view.board_to_screen(from);
                    let to_pos = view.board_to_screen(to);
                    
                    if let Some(pending) = self.game.pending_move {
                        self.animations.piece_move = Some(PieceMoveAnimation {
                            piece_id: self.game.board.piece_at(to.0, to.1)
                                .map(|p| p.id)
                                .unwrap_or(0),
                            from: from_pos,
                            to: to_pos,
                            start_time: Instant::now(),
                            duration_ms: PIECE_MOVE_DURATION_MS,
                            is_ai: pending.is_ai,
                        });
                    }
                }
                
                // 播放落子音效
                self.sound.place();
            }
            Err(e) => {
                eprintln!("AI选择移动失败: {}", e);
                // 如果AI移动失败，回到玩家回合
                let _ = self.game.handle_event(GameEvent::GameEndCheckComplete { result: None });
            }
        }
        
        self.ai_think_start = None;
    }

    /// 开始悔棋动画
    fn start_undo_animation(&mut self) {
        // 需要至少两步历史记录（AI一步 + 玩家一步）
        if self.game.move_history.len() < 2 {
            // 历史记录不足，直接完成悔棋
            let _ = self.game.handle_event(GameEvent::UndoAnimationComplete);
            return;
        }

        let view = match self.board_view {
            Some(ref v) => v.clone(),
            None => return,
        };

        // 获取最后两步记录
        let ai_record = self.game.move_history.last().cloned().unwrap();
        let player_record = self.game.move_history.iter().nth_back(1).cloned().unwrap();

        // 获取棋子当前位置
        let ai_piece_current_pos = if let Some(piece) = self.game.board.piece_by_id(ai_record.piece_id) {
            view.board_to_screen(piece.position)
        } else {
            let _ = self.game.handle_event(GameEvent::UndoAnimationComplete);
            return;
        };

        let player_piece_current_pos = if let Some(piece) = self.game.board.piece_by_id(player_record.piece_id) {
            view.board_to_screen(piece.position)
        } else {
            let _ = self.game.handle_event(GameEvent::UndoAnimationComplete);
            return;
        };

        // 计算目标位置（回退后的位置）
        let ai_target_pos = view.board_to_screen(ai_record.from);
        let player_target_pos = view.board_to_screen(player_record.from);

        // 准备被吃棋子的动画信息
        let captured_piece = if !ai_record.captured.is_empty() {
            let captured_record = &ai_record.captured[0];
            Some(CapturedPieceInfo {
                record: captured_record.clone(),
                screen_pos: view.board_to_screen(captured_record.position),
            })
        } else {
            None
        };

        // 创建悔棋动画
        self.animations.undo = Some(UndoAnimation {
            step: UndoStep::AiUndoing,
            ai_move: PieceMoveAnimation {
                piece_id: ai_record.piece_id,
                from: ai_piece_current_pos,
                to: ai_target_pos,
                start_time: Instant::now(),
                duration_ms: UNDO_STEP_DURATION_MS,
                is_ai: true,
            },
            player_move: PieceMoveAnimation {
                piece_id: player_record.piece_id,
                from: player_piece_current_pos,
                to: player_target_pos,
                start_time: Instant::now(), // 会在第三步更新
                duration_ms: UNDO_STEP_DURATION_MS,
                is_ai: false,
            },
            ai_record,
            player_record,
            captured_piece,
        });
    }

    /// 处理玩家输入
    fn handle_player_input(&mut self, _ctx: &Context, response: &egui::Response) {
        // 只有在等待玩家行棋或棋子吸附状态才能操作
        match self.game.state {
            GameState::WaitingForPlayer => {
                self.handle_waiting_input(response);
            }
            GameState::PieceDragging => {
                self.handle_dragging_input(response);
            }
            _ => {}
        }
    }

    /// 处理等待玩家行棋状态的输入
    fn handle_waiting_input(&mut self, response: &egui::Response) {
        let view = match self.board_view {
            Some(ref v) => v.clone(),
            None => return,
        };

        // 处理鼠标左键点击（进入棋子吸附状态）
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pos) = response.interact_pointer_pos() {
                // 查找点击的己方棋子
                let clicked_piece = self.game.board.active_pieces_of(self.game.player_side)
                    .into_iter()
                    .find(|piece| view.hit_test_piece(pos, piece.position));
                
                if let Some(piece) = clicked_piece {
                    // 检查棋子是否可以移动
                    if self.can_piece_move(piece.id) {
                        self.sound.click();
                        // 保存拖拽信息到临时存储
                        self.drag_info = Some(DragInfo {
                            piece_id: piece.id,
                            start_pos: piece.position,
                            current_mouse_pos: (pos.x, pos.y),
                        });
                        // 发送事件进入吸附状态
                        let _ = self.game.handle_event(GameEvent::PlayerStartDrag {
                            piece_id: piece.id,
                            start_pos: piece.position,
                        });
                    }
                }
            }
        }
    }

    /// 处理棋子吸附状态的输入
    fn handle_dragging_input(&mut self, response: &egui::Response) {
        let view = match self.board_view {
            Some(ref v) => v.clone(),
            None => return,
        };

        // 从游戏状态更新拖拽信息
        if let Some(ref drag) = self.game.drag_state {
            self.drag_info = Some(DragInfo {
                piece_id: drag.piece_id,
                start_pos: drag.start_pos,
                current_mouse_pos: drag.current_mouse_pos,
            });
        }

        // 更新吸附位置（鼠标移动时棋子跟随，不需要按住鼠标）
        if let Some(ref mut drag_info) = self.drag_info {
            if let Some(pos) = response.hover_pos() {
                // 限制在棋盘范围内
                let clamped_pos = egui::Pos2::new(
                    pos.x.clamp(view.rect.min.x, view.rect.max.x),
                    pos.y.clamp(view.rect.min.y, view.rect.max.y),
                );
                drag_info.current_mouse_pos = (clamped_pos.x, clamped_pos.y);
                
                // 同步更新游戏状态中的拖拽位置
                if let Some(ref mut drag) = self.game.drag_state {
                    drag.current_mouse_pos = (clamped_pos.x, clamped_pos.y);
                }
            }
        }

        // 处理右键取消（点击右键取消吸附）
        if response.clicked_by(egui::PointerButton::Secondary) {
            self.sound.place();
            
            if let Some(drag_info) = self.drag_info.take() {
                let _ = self.game.handle_event(GameEvent::PlayerCancel);
                
                // 触发放回原位动画
                let current_pos = egui::Pos2::new(drag_info.current_mouse_pos.0, drag_info.current_mouse_pos.1);
                let original_pos = view.board_to_screen(drag_info.start_pos);
                
                self.animations.piece_return = Some(PieceReturnAnimation {
                    piece_id: drag_info.piece_id,
                    from: current_pos,
                    to: original_pos,
                    start_time: Instant::now(),
                    duration_ms: PIECE_RETURN_DURATION_MS,
                });
            }
            return;
        }

        // 处理左键落子（点击左键放下棋子）
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(drag_info) = self.drag_info.take() {
                let current_pos = egui::Pos2::new(drag_info.current_mouse_pos.0, drag_info.current_mouse_pos.1);
                
                // 尝试转换到棋盘坐标
                if let Some(target_pos) = view.screen_to_board(current_pos, 0.4) {
                    // 发送落子事件
                    let _ = self.game.handle_event(GameEvent::PlayerDrop { target_pos });
                    
                    // 检查是否进入移动动画状态
                    if matches!(self.game.state, GameState::PieceMoving) {
                        let to_pos = view.board_to_screen(target_pos);
                        
                        self.animations.piece_move = Some(PieceMoveAnimation {
                            piece_id: drag_info.piece_id,
                            from: current_pos,
                            to: to_pos,
                            start_time: Instant::now(),
                            duration_ms: PIECE_MOVE_DURATION_MS,
                            is_ai: false,
                        });
                        
                        self.sound.place();
                    } else if matches!(self.game.state, GameState::PieceReturning) {
                        // 非法落点，放回原位
                        let original_pos = view.board_to_screen(drag_info.start_pos);
                        
                        self.animations.piece_return = Some(PieceReturnAnimation {
                            piece_id: drag_info.piece_id,
                            from: current_pos,
                            to: original_pos,
                            start_time: Instant::now(),
                            duration_ms: PIECE_RETURN_DURATION_MS,
                        });
                        
                        self.sound.invalid();
                    }
                } else {
                    // 超出容错范围，放回原位
                    let _ = self.game.handle_event(GameEvent::PlayerCancel);
                    let original_pos = view.board_to_screen(drag_info.start_pos);
                    
                    self.animations.piece_return = Some(PieceReturnAnimation {
                        piece_id: drag_info.piece_id,
                        from: current_pos,
                        to: original_pos,
                        start_time: Instant::now(),
                        duration_ms: PIECE_RETURN_DURATION_MS,
                    });
                    
                    self.sound.invalid();
                }
            }
        }
    }

    /// 检查指定棋子是否可以移动
    fn can_piece_move(&self, piece_id: u8) -> bool {
        if let Some(piece) = self.game.board.piece_by_id(piece_id) {
            if piece.side != self.game.player_side || !piece.active {
                return false;
            }

            let (x, y) = piece.position;
            let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

            for (dx, dy) in directions {
                let nx = x as i8 + dx;
                let ny = y as i8 + dy;

                if Board::is_valid_pos(nx, ny) && self.game.board.is_empty(nx as u8, ny as u8) {
                    return true;
                }
            }
        }
        false
    }

    /// 更新所有动画
    fn update_animations(&mut self) {
        // 更新棋子移动动画
        if let Some(ref anim) = self.animations.piece_move {
            let elapsed = anim.start_time.elapsed().as_millis() as u64;
            if elapsed >= anim.duration_ms {
                // 动画完成
                let moved = anim.from != anim.to;
                let _ = self.game.handle_event(GameEvent::PieceMoveAnimationComplete { moved });
                
                // 检查是否产生了吃子
                if moved && !self.game.last_captured.is_empty() {
                    self.animations.capture = Some(CaptureAnimation {
                        piece_ids: self.game.last_captured.clone(),
                        start_time: Instant::now(),
                        stage: CaptureStage::Flashing,
                    });
                    self.sound.capture();
                }
                
                self.animations.piece_move = None;
            }
        }

        // 更新棋子放回原位动画
        if let Some(ref anim) = self.animations.piece_return {
            let elapsed = anim.start_time.elapsed().as_millis() as u64;
            if elapsed >= anim.duration_ms {
                let _ = self.game.handle_event(GameEvent::PieceReturnAnimationComplete);
                self.animations.piece_return = None;
            }
        }

        // 更新吃子动画
        if let Some(ref mut anim) = self.animations.capture {
            let elapsed = anim.start_time.elapsed().as_millis() as u64;
            
            match anim.stage {
                CaptureStage::Flashing if elapsed >= CAPTURE_FLASH_DURATION_MS => {
                    anim.stage = CaptureStage::Removing;
                    anim.start_time = Instant::now();
                }
                CaptureStage::Removing if elapsed >= CAPTURE_REMOVE_DURATION_MS => {
                    let _ = self.game.handle_event(GameEvent::CaptureAnimationComplete);
                    self.animations.capture = None;
                }
                _ => {}
            }
        }

        // 更新悔棋动画
        if let Some(ref mut anim) = self.animations.undo {
            let now = Instant::now();
            
            match anim.step {
                UndoStep::AiUndoing => {
                    let elapsed = now.duration_since(anim.ai_move.start_time).as_millis() as u64;
                    if elapsed >= anim.ai_move.duration_ms {
                        if anim.captured_piece.is_some() {
                            anim.step = UndoStep::CapturedReturning;
                        } else {
                            // 没有被吃棋子，直接进入第三步，更新玩家动画开始时间
                            anim.player_move.start_time = now;
                            anim.step = UndoStep::PlayerUndoing;
                        }
                    }
                }
                UndoStep::CapturedReturning => {
                    let ai_end = anim.ai_move.start_time + Duration::from_millis(anim.ai_move.duration_ms);
                    let elapsed = now.duration_since(ai_end).as_millis() as u64;
                    if elapsed >= UNDO_STEP_DURATION_MS {
                        // 进入第三步时更新玩家动画的开始时间
                        anim.player_move.start_time = now;
                        anim.step = UndoStep::PlayerUndoing;
                    }
                }
                UndoStep::PlayerUndoing => {
                    let elapsed = now.duration_since(anim.player_move.start_time).as_millis() as u64;
                    if elapsed >= anim.player_move.duration_ms {
                        let _ = self.game.handle_event(GameEvent::UndoAnimationComplete);
                        self.animations.undo = None;
                    }
                }
            }
        }
    }

    /// 处理状态流转（非动画驱动的事件）
    fn process_state_transitions(&mut self) {
        match self.game.state {
            GameState::NewGame => {
                // 新局开始后自动流转到下一状态
                if self.game.player_side == self.game.current_turn {
                    let _ = self.game.handle_event(GameEvent::StartNewGame { player_first: true });
                } else {
                    let _ = self.game.handle_event(GameEvent::StartNewGame { player_first: false });
                }
            }
            GameState::UndoAnimating if self.animations.undo.is_none() => {
                // 进入悔棋动画状态，需要创建动画
                self.start_undo_animation();
            }
            GameState::CheckingCapture => {
                let has_capture = !self.game.last_captured.is_empty();
                let captured = self.game.last_captured.clone();
                let _ = self.game.handle_event(GameEvent::CaptureCheckComplete { 
                    has_capture, 
                    captured_piece_ids: captured 
                });
            }
            GameState::CheckingGameEnd => {
                let result = self.game.check_game_end();
                
                // 检查是否需要切换回合后再检查困毙（AI移动后需要检查人类方）
                let final_result = if result.is_none() {
                    // 先发送事件给状态机处理（这会切换回合）
                    let _ = self.game.handle_event(GameEvent::GameEndCheckComplete { result });
                    // 切换回合后，检查新回合方是否被困毙
                    self.game.check_stalemate_for_current_turn()
                } else {
                    // 已经有结果（无子判负或平局），直接发送事件
                    let _ = self.game.handle_event(GameEvent::GameEndCheckComplete { result });
                    result
                };
                
                // 如果游戏结束，播放相应音效并显示对话框
                if let Some(final_result) = final_result {
                    match final_result {
                        GameResult::PlayerWin => self.sound.win(),
                        GameResult::AiWin => self.sound.lose(),
                        GameResult::Draw => self.sound.draw(),
                    }
                    self.game_over_dialog = GameOverDialog::Open(final_result);
                }
            }
            _ => {}
        }
    }

    /// 渲染游戏画面
    fn render_game(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let board_size = available_size.min_elem().min(500.0);
        let center = ui.available_rect_before_wrap().center();

        // 根据玩家执子方决定是否翻转棋盘
        let flip = self.game.player_side == Side::White;
        let view = BoardView::new(center, board_size, flip, ui.ctx());

        // 绘制棋盘
        let response = view.draw_board(ui);

        // 绘制原始位置标记（当棋子被吸附时）
        if let GameState::PieceDragging = self.game.state {
            if let Some(ref drag) = self.game.drag_state {
                view.draw_origin_marker(ui, drag.start_pos);
            }
        }

        // 收集悔棋动画中需要显示的被吃棋子ID
        let undo_captured_id = self.animations.undo.as_ref()
            .and_then(|u| u.captured_piece.as_ref())
            .map(|c| c.record.piece_id);

        // 绘制所有棋子
        for piece in &self.game.board.pieces {
            let is_captured_in_undo = undo_captured_id == Some(piece.id);

            if !piece.active && !is_captured_in_undo {
                continue;
            }

            // 检查是否是正在拖拽的棋子
            let is_dragging = matches!(self.game.state, GameState::PieceDragging)
                && self.game.drag_state.as_ref().map(|d| d.piece_id) == Some(piece.id);

            if is_dragging {
                if let Some(ref drag) = self.game.drag_state {
                    let pos = egui::Pos2::new(drag.current_mouse_pos.0, drag.current_mouse_pos.1);
                    view.draw_dragging_piece(ui, piece, pos);
                }
            } else if let Some(ref anim) = self.animations.piece_move {
                // 移动动画中
                if anim.piece_id == piece.id {
                    let elapsed = anim.start_time.elapsed().as_millis() as f64;
                    let progress = (elapsed / anim.duration_ms as f64).min(1.0);
                    let t = crate::utils::ease_in_out_quad(progress as f32);

                    let current_pos = egui::Pos2::new(
                        crate::utils::lerp(anim.from.x, anim.to.x, t),
                        crate::utils::lerp(anim.from.y, anim.to.y, t),
                    );

                    view.draw_animated_piece(ui, piece, current_pos);
                } else {
                    view.draw_piece(ui, piece, false, None);
                }
            } else if let Some(ref anim) = self.animations.piece_return {
                // 放回原位动画中
                if anim.piece_id == piece.id {
                    let elapsed = anim.start_time.elapsed().as_millis() as f64;
                    let progress = (elapsed / anim.duration_ms as f64).min(1.0);
                    let t = crate::utils::ease_out_bounce(progress as f32);

                    let current_pos = egui::Pos2::new(
                        crate::utils::lerp(anim.from.x, anim.to.x, t),
                        crate::utils::lerp(anim.from.y, anim.to.y, t),
                    );

                    view.draw_animated_piece(ui, piece, current_pos);
                } else {
                    view.draw_piece(ui, piece, false, None);
                }
            } else if let Some(ref undo) = self.animations.undo {
                // 悔棋动画中
                self.render_undo_animation_piece(ui, &view, piece, undo);
            } else {
                view.draw_piece(ui, piece, false, None);
            }
        }

        // 绘制吃子动画
        self.render_capture_animation(ui, &view);

        self.board_view = Some(view);
        self.handle_player_input(ui.ctx(), &response);
    }

    /// 渲染悔棋动画中的棋子
    fn render_undo_animation_piece(&self, ui: &mut egui::Ui, view: &BoardView, piece: &crate::game::piece::Piece, undo: &UndoAnimation) {
        let is_ai_piece = piece.id == undo.ai_move.piece_id;
        let is_player_piece = undo.player_move.piece_id == piece.id;
        let is_captured_piece = undo.captured_piece.as_ref().map(|c| c.record.piece_id) == Some(piece.id);

        if is_ai_piece {
            // AI棋子回退动画
            let elapsed = undo.ai_move.start_time.elapsed().as_millis() as f64;
            let progress = (elapsed / undo.ai_move.duration_ms as f64).min(1.0);
            let t = crate::utils::ease_out_quad(progress as f32);

            let current_pos = egui::Pos2::new(
                crate::utils::lerp(undo.ai_move.from.x, undo.ai_move.to.x, t),
                crate::utils::lerp(undo.ai_move.from.y, undo.ai_move.to.y, t),
            );

            view.draw_animated_piece(ui, piece, current_pos);
        } else if is_captured_piece {
            // 被吃棋子的动画
            match undo.step {
                UndoStep::AiUndoing => {
                    // 渐显
                    let elapsed = undo.ai_move.start_time.elapsed().as_millis() as f64;
                    let progress = (elapsed / undo.ai_move.duration_ms as f64).min(1.0);
                    let alpha = (progress * 255.0) as u8;

                    if let Some(ref captured) = undo.captured_piece {
                        view.draw_piece_with_alpha(ui, piece, captured.screen_pos, alpha);
                    }
                }
                UndoStep::CapturedReturning => {
                    // 回退
                    let ai_end = undo.ai_move.start_time + Duration::from_millis(undo.ai_move.duration_ms);
                    let elapsed = std::time::Instant::now().duration_since(ai_end).as_millis() as f64;
                    let progress = (elapsed / UNDO_STEP_DURATION_MS as f64).min(1.0);
                    let t = crate::utils::ease_out_quad(progress as f32);

                    if let Some(ref captured) = undo.captured_piece {
                        let target_pos = view.board_to_screen(undo.player_record.from);
                        let current_pos = egui::Pos2::new(
                            crate::utils::lerp(captured.screen_pos.x, target_pos.x, t),
                            crate::utils::lerp(captured.screen_pos.y, target_pos.y, t),
                        );
                        view.draw_animated_piece(ui, piece, current_pos);
                    }
                }
                UndoStep::PlayerUndoing => {
                    view.draw_piece(ui, piece, false, None);
                }
            }
        } else if is_player_piece && matches!(undo.step, UndoStep::PlayerUndoing) {
            // 玩家棋子回退动画
            let elapsed = undo.player_move.start_time.elapsed().as_millis() as f64;
            let progress = (elapsed / undo.player_move.duration_ms as f64).min(1.0);
            let t = crate::utils::ease_out_quad(progress as f32);

            let current_pos = egui::Pos2::new(
                crate::utils::lerp(undo.player_move.from.x, undo.player_move.to.x, t),
                crate::utils::lerp(undo.player_move.from.y, undo.player_move.to.y, t),
            );

            view.draw_animated_piece(ui, piece, current_pos);
        } else {
            view.draw_piece(ui, piece, false, None);
        }
    }

    /// 渲染吃子动画
    fn render_capture_animation(&mut self, ui: &mut egui::Ui, view: &BoardView) {
        if let Some(ref anim) = self.animations.capture {
            let elapsed = anim.start_time.elapsed().as_millis() as u64;

            match anim.stage {
                CaptureStage::Flashing => {
                    // 闪烁阶段
                    let flash_count = 3;
                    let flash_duration = CAPTURE_FLASH_DURATION_MS / flash_count;
                    let flash_progress = (elapsed % flash_duration) as f32 / flash_duration as f32;
                    let visible = flash_progress < 0.5;

                    if visible {
                        for &piece_id in &anim.piece_ids {
                            if let Some(piece) = self.game.board.piece_by_id(piece_id) {
                                view.draw_piece(ui, piece, false, None);
                            }
                        }
                    }
                }
                CaptureStage::Removing => {
                    // 移除阶段
                    let progress = (elapsed as f32 / CAPTURE_REMOVE_DURATION_MS as f32).min(1.0);

                    for &piece_id in &anim.piece_ids {
                        if let Some(piece) = self.game.board.piece_by_id(piece_id) {
                            view.draw_capturing_piece(ui, piece, progress);
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 处理菜单（根据当前状态决定是否可操作）
        self.handle_menu(ctx);
        self.handle_toolbar(ctx);

        // 处理对话框
        self.handle_new_game_dialog(ctx);
        self.handle_game_over_dialog(ctx);
        self.about_dialog.show(ctx);
        self.rules_dialog.show(ctx);

        // 处理加载确认对话框
        if self.pending_load_file.is_some() {
            self.show_confirm_load_dialog(ctx);
        }

        // 处理覆盖确认对话框
        if self.confirm_overwrite {
            self.show_confirm_overwrite_dialog(ctx);
        }

        // 处理AI回合
        if matches!(self.game.state, GameState::AiThinking) {
            self.handle_ai_turn();
        }

        // 处理状态流转
        self.process_state_transitions();

        // 更新动画
        self.update_animations();

        // 主面板
        CentralPanel::default().show(ctx, |ui| {
            self.render_game(ui);
        });

        // 请求连续更新以支持动画
        if self.has_active_animation()
            || matches!(self.game.state, GameState::AiThinking)
            || matches!(self.game.state, GameState::CheckingCapture)
            || matches!(self.game.state, GameState::CheckingGameEnd)
        {
            ctx.request_repaint();
        }
    }
}
