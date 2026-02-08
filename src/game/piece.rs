//! 棋子定义

use serde::{Deserialize, Serialize};

/// 棋子颜色（方）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    /// 黑方
    Black,
    /// 白方
    White,
}

impl Side {
    /// 获取对方
    pub fn opposite(&self) -> Self {
        match self {
            Side::Black => Side::White,
            Side::White => Side::Black,
        }
    }

    /// 黑方先行
    pub fn first() -> Self {
        Side::Black
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Black => write!(f, "黑方"),
            Side::White => write!(f, "白方"),
        }
    }
}

/// 棋子状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PieceState {
    #[default]
    /// 静止
    Idle,
    /// 正在被拖拽
    Dragging,
    /// 动画中
    Animating,
    /// 被吃掉的动画中
    BeingCaptured,
}

/// 棋子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Piece {
    /// 棋子唯一ID
    pub id: u8,
    /// 所属方
    pub side: Side,
    /// 当前位置 (x, y)
    pub position: (u8, u8),
    /// 当前状态
    #[serde(skip)]
    pub state: PieceState,
    /// 是否仍在棋盘上
    pub active: bool,
}

impl Piece {
    /// 创建新棋子
    pub fn new(id: u8, side: Side, x: u8, y: u8) -> Self {
        Self {
            id,
            side,
            position: (x, y),
            state: PieceState::Idle,
            active: true,
        }
    }

    /// 获取显示名称
    pub fn name(&self) -> String {
        format!("{}{}", self.side, self.id)
    }
}

/// 获取初始棋子布局
pub fn initial_pieces() -> Vec<Piece> {
    let mut pieces = Vec::with_capacity(12);
    let mut id = 1u8;

    // 黑方初始位置
    let black_positions = [(0, 0), (1, 0), (2, 0), (3, 0), (0, 1), (3, 1)];
    for (x, y) in black_positions {
        pieces.push(Piece::new(id, Side::Black, x, y));
        id += 1;
    }

    // 白方初始位置
    let white_positions = [(0, 3), (1, 3), (2, 3), (3, 3), (0, 2), (3, 2)];
    for (x, y) in white_positions {
        pieces.push(Piece::new(id, Side::White, x, y));
        id += 1;
    }

    pieces
}
