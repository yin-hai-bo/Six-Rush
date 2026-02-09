//! 游戏状态机定义
//!
//! 按照 specification.md 中的状态流转图实现

use serde::{Deserialize, Serialize};

/// 游戏状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    /// 新局开始 - 初始化棋盘，决定先行方
    NewGame,
    
    /// 电脑思考中 - AI计算行棋方案
    /// 此状态下玩家不可操作任何UI控件
    AiThinking,
    
    /// 等待玩家行棋 - 玩家可操作UI，可点击棋子或悔棋
    /// 此为"初始状态"，从此状态开始交互
    WaitingForPlayer,
    
    /// 初始吸附状态 - 玩家左键按下棋子后吸附状态
    /// 此状态下：
    /// - 棋子吸附在鼠标上
    /// - 高亮显示原始位置
    /// - 标注所有合法目标点
    PieceSelected,
    
    /// 拖拽状态 - 玩家在初始吸附状态下移动鼠标（按住左键）
    /// 此状态下：
    /// - 棋子跟随鼠标移动
    /// - 保持高亮原始位置
    /// - 保持标注合法目标点
    PieceDragging,
    
    /// 待点击目标点状态 - 玩家在初始吸附状态下松开左键（未移动）
    /// 此状态下：
    /// - 棋子放回原始位置
    /// - 保持高亮原始位置
    /// - 保持标注合法目标点
    /// - 等待玩家点击目标点
    WaitingForTargetClick,
    
    /// 棋子移动动画 - 棋子以动画方式移动到目标位置
    PieceMoving,
    
    /// 棋子放回原位 - 非法移动或右键取消，棋子动画回到原位
    PieceReturning,
    
    /// 判断吃子 - 程序判断是否产生吃子
    CheckingCapture,
    
    /// 吃子动画 - 被吃棋子闪烁、消失
    CaptureAnimating,
    
    /// 胜负判断 - 判断是否出现胜负或平局
    CheckingGameEnd,
    
    /// 胜负平局弹框 - 显示结果对话框
    /// 此状态下可操作UI（悔棋、新局）
    GameOverDialog(GameResult),
    
    /// 悔棋动画中 - 棋子回退动画
    UndoAnimating,
}

/// 游戏结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    /// 玩家获胜
    PlayerWin,
    /// 电脑获胜
    AiWin,
    /// 平局
    Draw,
}

impl GameResult {
    /// 获取本地化的显示文本
    pub fn display_text(&self) -> String {
        match self {
            GameResult::PlayerWin => crate::t!("game.player_win"),
            GameResult::AiWin => crate::t!("game.ai_win"),
            GameResult::Draw => crate::t!("game.draw"),
        }
    }
}

impl GameState {
    /// 检查当前状态是否可操作UI
    /// 
    /// 根据 spec:
    /// - 可操作UI的状态: 等待玩家行棋、胜负平局弹框、待点击目标点状态
    /// - 不可操作UI的状态: 其他所有状态
    pub fn can_interact_with_ui(&self) -> bool {
        matches!(self, 
            GameState::WaitingForPlayer | 
            GameState::WaitingForTargetClick |
            GameState::GameOverDialog(_)
        )
    }
    
    /// 检查当前状态是否可以悔棋
    pub fn can_undo(&self) -> bool {
        matches!(self, GameState::WaitingForPlayer | GameState::WaitingForTargetClick)
    }
    
    /// 检查当前状态是否可以点击棋子
    pub fn can_select_piece(&self) -> bool {
        matches!(self, GameState::WaitingForPlayer)
    }
    
    /// 检查当前是否处于动画状态
    pub fn is_animating(&self) -> bool {
        matches!(self,
            GameState::PieceMoving |
            GameState::PieceReturning |
            GameState::CaptureAnimating |
            GameState::UndoAnimating
        )
    }
    
    /// 检查当前状态是否需要AI行动
    pub fn needs_ai_move(&self) -> bool {
        matches!(self, GameState::AiThinking)
    }
}

/// 拖拽状态
#[derive(Debug, Clone, Copy)]
pub struct DragState {
    /// 拖拽的棋子ID
    pub piece_id: u8,
    /// 起始位置（棋盘坐标）
    pub start_pos: (u8, u8),
    /// 当前鼠标位置（屏幕坐标）
    pub current_mouse_pos: (f32, f32),
}

/// 动画类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    /// 棋子移动
    PieceMove,
    /// 吃子动画
    Capture,
    /// 悔棋动画
    Undo,
}

/// 游戏流转事件
/// 
/// 用于驱动状态机从一个状态流转到另一个状态
#[derive(Debug, Clone)]
pub enum GameEvent {
    /// 开始新局
    StartNewGame { player_first: bool },
    /// AI思考完成，选定落点
    AiMoveSelected { from: (u8, u8), to: (u8, u8) },
    /// 玩家点击棋子开始吸附（左键DOWN）
    PlayerStartDrag { piece_id: u8, start_pos: (u8, u8) },
    /// 玩家在初始吸附状态下移动鼠标，进入拖拽状态
    PlayerStartMoving,
    /// 玩家在初始吸附状态下松开左键（未移动），进入待点击目标点状态
    PlayerReleaseWithoutMove,
    /// 玩家在待点击目标点状态下点击目标点
    PlayerClickTarget { target_pos: (u8, u8) },
    /// 玩家左键落子（拖拽状态下）
    PlayerDrop { target_pos: (u8, u8) },
    /// 玩家右键取消（放回原地）
    PlayerCancel,
    /// 玩家点击非目标点位置，返回初始状态
    PlayerClickInvalid,
    /// 棋子移动动画完成
    PieceMoveAnimationComplete { moved: bool },
    /// 棋子放回原位动画完成
    PieceReturnAnimationComplete,
    /// 吃子检查完成
    CaptureCheckComplete { has_capture: bool, captured_piece_ids: Vec<u8> },
    /// 吃子动画完成
    CaptureAnimationComplete,
    /// 胜负判断完成
    GameEndCheckComplete { result: Option<GameResult> },
    /// 点击对话框按钮
    DialogAction(DialogAction),
    /// 开始悔棋
    StartUndo,
    /// 悔棋动画完成
    UndoAnimationComplete,
}

/// 对话框操作
#[derive(Debug, Clone, Copy)]
pub enum DialogAction {
    /// 悔棋
    Undo,
    /// 开始新局
    NewGame,
    /// 确定/返回
    Confirm,
}

/// 移动结果
#[derive(Debug, Clone)]
pub struct MoveResult {
    /// 是否成功移动
    pub moved: bool,
    /// 被吃掉的棋子ID列表
    pub captured: Vec<u8>,
    /// 是否放回原位
    pub returned: bool,
}
