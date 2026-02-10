//! 游戏核心逻辑模块

pub mod ai;
pub mod audio;
pub mod board;
pub mod piece;
pub mod rules;
pub mod save;
pub mod state;

use crate::game::board::Board;
use crate::game::piece::Side;
use crate::game::rules::{check_game_end, calculate_captures};
use crate::game::state::GameEvent;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 被吃棋子的记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRecord {
    /// 棋子ID
    pub piece_id: u8,
    /// 被吃掉时的位置
    pub position: (u8, u8),
}

/// 移动记录（用于悔棋）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRecord {
    /// 移动的棋子ID
    pub piece_id: u8,
    /// 起始位置
    pub from: (u8, u8),
    /// 目标位置
    pub to: (u8, u8),
    /// 被吃掉的棋子记录（包含ID和位置）
    pub captured: Vec<CapturedRecord>,
    /// 是否进入了单子状态（用于规则恢复）
    pub was_single_piece_mode: bool,
    /// 行棋方
    pub side: Side,
}

/// 游戏主结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    /// 当前棋盘状态
    pub board: Board,
    /// 当前游戏状态
    pub state: GameState,
    /// 玩家执子方（黑/白）
    pub player_side: Side,
    /// 当前轮到哪方行棋
    pub current_turn: Side,
    /// 行棋历史（用于悔棋）
    pub move_history: Vec<MoveRecord>,
    /// AI难度等级 (1-5)
    pub ai_level: u8,
    /// 当前选中的棋子（仅在PieceSelected状态下有效）
    #[serde(skip)]
    pub selected_piece: Option<SelectedPiece>,
    /// 当前正在移动的棋子动画信息
    #[serde(skip)]
    pub pending_move: Option<PendingMove>,
    /// 最近一次被吃掉的棋子ID列表（用于动画）
    #[serde(skip)]
    pub last_captured: Vec<u8>,
    /// 游戏结果（如果已结束）
    pub last_result: Option<GameResult>,
}

/// 待执行的移动（用于动画）
#[derive(Debug, Clone, Copy)]
pub struct PendingMove {
    pub from: (u8, u8),
    pub to: (u8, u8),
    pub is_ai: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: Board::default(),
            state: GameState::NewGame,
            player_side: Side::Black,
            current_turn: Side::Black,
            move_history: Vec::new(),
            ai_level: 3,
            selected_piece: None,
            pending_move: None,
            last_captured: Vec::new(),
            last_result: None,
        }
    }
}

impl Game {
    /// 创建新游戏
    pub fn new() -> Self {
        Self::default()
    }

    /// 处理游戏事件，驱动状态机流转
    /// 
    /// 这是状态机的核心方法，根据当前状态和事件决定下一个状态
    pub fn handle_event(&mut self, event: GameEvent) -> Result<()> {
        match (&self.state, event) {
            // ===== 新局开始 =====
            (GameState::NewGame, GameEvent::StartNewGame { player_first, ai_level }) => {
                self.start_new_game(player_first, ai_level);
            }
            
            // 电脑先行 -> 进入电脑思考中
            (GameState::NewGame, _) if self.current_turn != self.player_side => {
                self.state = GameState::AiThinking;
            }
            
            // 玩家先行 -> 等待玩家行棋
            (GameState::NewGame, _) => {
                self.state = GameState::WaitingForPlayer;
            }
            
            // ===== 等待玩家行棋（初始状态）=====
            (GameState::WaitingForPlayer, GameEvent::PlayerSelectPiece { piece_id, start_pos }) => {
                // 检查是否是己方棋子且有可移动位置
                if self.can_piece_move(piece_id) {
                    self.selected_piece = Some(SelectedPiece {
                        piece_id,
                        start_pos,
                    });
                    // 进入棋子已选中状态
                    self.state = GameState::PieceSelected;
                }
            }
            
            (GameState::WaitingForPlayer, GameEvent::StartUndo) => {
                if self.can_undo() {
                    self.state = GameState::UndoAnimating;
                }
            }
            
            // ===== 棋子已选中状态 =====
            (GameState::PieceSelected, GameEvent::PlayerClickTarget { target_pos }) => {
                if let Some(selected) = self.selected_piece {
                    // 执行移动
                    self.pending_move = Some(PendingMove {
                        from: selected.start_pos,
                        to: target_pos,
                        is_ai: false,
                    });
                    self.state = GameState::PieceMoving;
                    self.selected_piece = None;
                }
            }
            
            // 点击了非目标点或右键，返回初始状态
            (GameState::PieceSelected, GameEvent::PlayerClickInvalid) |
            (GameState::PieceSelected, GameEvent::PlayerCancel) => {
                self.selected_piece = None;
                self.state = GameState::WaitingForPlayer;
            }
            
            // ===== 棋子移动动画 =====
            (GameState::PieceMoving, GameEvent::PieceMoveAnimationComplete { moved }) => {
                if let Some(pending) = self.pending_move {
                    if moved {
                        // 执行实际的移动
                        let record = self.execute_move(pending.from, pending.to, self.player_side)?;
                        self.last_captured = record.captured.iter().map(|c| c.piece_id).collect();
                        self.move_history.push(record);
                        
                        // 进入判断吃子状态
                        self.state = GameState::CheckingCapture;
                    } else {
                        self.state = GameState::WaitingForPlayer;
                    }
                    self.pending_move = None;
                }
            }
            

            // ===== 判断吃子 =====
            (GameState::CheckingCapture, GameEvent::CaptureCheckComplete { has_capture, .. }) => {
                if has_capture {
                    self.state = GameState::CaptureAnimating;
                } else {
                    self.state = GameState::CheckingGameEnd;
                }
            }
            
            // ===== 吃子动画 =====
            (GameState::CaptureAnimating, GameEvent::CaptureAnimationComplete) => {
                self.state = GameState::CheckingGameEnd;
            }
            
            // ===== 胜负判断 =====
            (GameState::CheckingGameEnd, GameEvent::GameEndCheckComplete { result }) => {
                if let Some(result) = result {
                    self.last_result = Some(result);
                    self.state = GameState::GameOverDialog(result);
                } else {
                    // 切换回合
                    self.current_turn = self.current_turn.opposite();
                    
                    // 切换回合后，检查新回合方是否被困毙
                    // 注意：这里需要检查新回合方（current_turn）是否有合法移动
                    if let Some(stalemate_result) = self.check_stalemate_for_current_turn() {
                        self.last_result = Some(stalemate_result);
                        self.state = GameState::GameOverDialog(stalemate_result);
                    } else {
                        // 根据当前轮到谁决定下一状态
                        if self.current_turn == self.player_side {
                            self.state = GameState::WaitingForPlayer;
                        } else {
                            self.state = GameState::AiThinking;
                        }
                    }
                }
            }
            
            // ===== 胜负平局弹框 =====
            (GameState::GameOverDialog(_), GameEvent::DialogAction(action)) => {
                match action {
                    DialogAction::Undo => {
                        if self.can_undo() {
                            self.state = GameState::UndoAnimating;
                        }
                    }
                    DialogAction::NewGame => {
                        self.state = GameState::NewGame;
                    }
                    DialogAction::Confirm => {
                        // 确定结束，保持相同先行方开启新局
                        let player_first = self.player_side == Side::Black;
                        self.start_new_game(player_first, self.ai_level);
                    }
                }
            }
            
            // ===== 电脑思考中 =====
            (GameState::AiThinking, GameEvent::AiMoveSelected { from, to }) => {
                self.pending_move = Some(PendingMove {
                    from,
                    to,
                    is_ai: true,
                });
                self.state = GameState::PieceMoving;
            }
            
            // ===== 悔棋动画 =====
            (GameState::UndoAnimating, GameEvent::UndoAnimationComplete) => {
                self.perform_undo()?;
                self.state = GameState::WaitingForPlayer;
            }
            
            // 其他未处理的事件组合
            _ => {}
        }
        
        Ok(())
    }
    
    /// 开始新局
    fn start_new_game(&mut self, player_first: bool, ai_level: u8) {
        self.board = Board::initial();
        self.player_side = if player_first { Side::Black } else { Side::White };
        self.current_turn = Side::Black; // 黑方先行
        self.move_history.clear();
        self.selected_piece = None;
        self.pending_move = None;
        self.last_captured.clear();
        self.last_result = None;
        self.ai_level = ai_level.clamp(1, 5);
        
        // 根据先行方设置初始状态
        if player_first {
            self.state = GameState::WaitingForPlayer;
        } else {
            self.state = GameState::AiThinking;
        }
    }
    
    /// 执行移动
    fn execute_move(&mut self, from: (u8, u8), to: (u8, u8), side: Side) -> Result<MoveRecord> {
        let was_single = self.board.is_single_piece_mode();
        
        let piece = self.board
            .piece_at_mut(from.0, from.1)
            .ok_or_else(|| anyhow::anyhow!("起始位置没有棋子"))?;
        
        let piece_id = piece.id;
        piece.position = to;
        
        // 检查吃子
        let captured_ids = calculate_captures(&self.board, piece_id);
        
        // 收集被吃棋子的记录
        let mut captured_records = Vec::new();
        for &captured_id in &captured_ids {
            if let Some(p) = self.board.piece_by_id(captured_id) {
                captured_records.push(CapturedRecord {
                    piece_id: captured_id,
                    position: p.position,
                });
            }
            if let Some(p) = self.board.piece_by_id_mut(captured_id) {
                p.active = false;
            }
        }
        
        Ok(MoveRecord {
            piece_id,
            from,
            to,
            captured: captured_records,
            was_single_piece_mode: was_single,
            side,
        })
    }
    
    /// 检查指定棋子是否可以移动
    fn can_piece_move(&self, piece_id: u8) -> bool {
        if let Some(piece) = self.board.piece_by_id(piece_id) {
            if piece.side != self.player_side || !piece.active {
                return false;
            }
            
            // 检查四个方向是否有可移动位置
            let (x, y) = piece.position;
            let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
            
            for (dx, dy) in directions {
                let nx = x as i8 + dx;
                let ny = y as i8 + dy;
                
                if Board::is_valid_pos(nx, ny) && self.board.is_empty(nx as u8, ny as u8) {
                    return true;
                }
            }
        }
        false
    }
    
    /// 检查是否可以悔棋
    /// 
    /// 根据 spec:
    /// - 在"等待玩家行棋"状态可以悔棋
    /// - 需要至少有一次历史记录
    pub fn can_undo(&self) -> bool {
        self.state.can_undo() && !self.move_history.is_empty()
    }
    
    /// 执行悔棋（实际修改棋盘状态）
    fn perform_undo(&mut self) -> Result<()> {
        // 需要回退两步（AI一步 + 玩家一步）
        for _ in 0..2 {
            if let Some(record) = self.move_history.pop() {
                self.board.undo_move(&record)?;
                self.current_turn = self.current_turn.opposite();
            } else {
                break;
            }
        }
        
        // 确保回到玩家回合
        self.current_turn = self.player_side;
        self.last_result = None;
        
        Ok(())
    }
    
    /// 检查游戏是否结束
    pub fn check_game_end(&self) -> Option<GameResult> {
        check_game_end(&self.board, self.current_turn, self.player_side)
    }
    
    /// 检查当前回合方是否被困毙
    /// 返回 Some(GameResult) 如果当前方被困毙，否则返回 None
    pub fn check_stalemate_for_current_turn(&self) -> Option<GameResult> {
        use crate::game::rules::is_stalemated;
        
        if is_stalemated(&self.board, self.current_turn) {
            // 当前方被困毙，判负，对方获胜
            let winner = self.current_turn.opposite();
            Some(if winner == self.player_side {
                GameResult::PlayerWin
            } else {
                GameResult::AiWin
            })
        } else {
            None
        }
    }
    
    /// 执行AI移动（由外部AI模块调用）
    pub fn execute_ai_move(&mut self, from: (u8, u8), to: (u8, u8)) -> Result<Vec<u8>> {
        let record = self.execute_move(from, to, self.player_side.opposite())?;
        let captured: Vec<u8> = record.captured.iter().map(|c| c.piece_id).collect();
        self.move_history.push(record);
        Ok(captured)
    }
}

// 重新导出状态相关的类型
pub use state::{AnimationType, DialogAction, GameResult, GameState, MoveResult, SelectedPiece};
