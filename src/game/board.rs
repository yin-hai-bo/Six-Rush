//! 棋盘定义与操作

use crate::game::piece::{initial_pieces, Piece, Side};
use crate::game::{CapturedRecord, MoveRecord};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 棋盘大小（4x4）
pub const BOARD_SIZE: u8 = 4;

/// 棋盘
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// 所有棋子
    pub pieces: Vec<Piece>,
}

impl Default for Board {
    fn default() -> Self {
        Self::empty()
    }
}

impl Board {
    /// 创建空棋盘
    pub fn empty() -> Self {
        Self { pieces: Vec::new() }
    }

    /// 创建初始棋盘
    pub fn initial() -> Self {
        Self {
            pieces: initial_pieces(),
        }
    }

    /// 获取指定位置的棋子（如果有）
    pub fn piece_at(&self, x: u8, y: u8) -> Option<&Piece> {
        self.pieces.iter().find(|p| p.active && p.position == (x, y))
    }

    /// 获取指定位置的棋子可变引用
    pub fn piece_at_mut(&mut self, x: u8, y: u8) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|p| p.active && p.position == (x, y))
    }

    /// 获取指定ID的棋子
    pub fn piece_by_id(&self, id: u8) -> Option<&Piece> {
        self.pieces.iter().find(|p| p.id == id)
    }

    /// 获取指定ID的棋子可变引用
    pub fn piece_by_id_mut(&mut self, id: u8) -> Option<&mut Piece> {
        self.pieces.iter_mut().find(|p| p.id == id)
    }

    /// 检查位置是否在棋盘内
    pub fn is_valid_pos(x: i8, y: i8) -> bool {
        x >= 0 && x < BOARD_SIZE as i8 && y >= 0 && y < BOARD_SIZE as i8
    }

    /// 检查位置是否为空
    pub fn is_empty(&self, x: u8, y: u8) -> bool {
        self.piece_at(x, y).is_none()
    }

    /// 获取某方的所有活跃棋子
    pub fn active_pieces_of(&self, side: Side) -> Vec<&Piece> {
        self.pieces
            .iter()
            .filter(|p| p.active && p.side == side)
            .collect()
    }

    /// 获取某方活跃棋子的数量
    pub fn count_active(&self, side: Side) -> usize {
        self.pieces
            .iter()
            .filter(|p| p.active && p.side == side)
            .count()
    }

    /// 检查是否为单子状态（某方只剩1枚棋子）
    pub fn is_single_piece_mode(&self) -> bool {
        self.count_active(Side::Black) == 1 || self.count_active(Side::White) == 1
    }

    /// 执行移动
    pub fn execute_move(&mut self, from: (u8, u8), to: (u8, u8), side: crate::game::piece::Side) -> Result<MoveRecord> {
        let was_single = self.is_single_piece_mode();
        
        let piece = self
            .piece_at_mut(from.0, from.1)
            .ok_or_else(|| anyhow::anyhow!("起始位置没有棋子"))?;
        
        let piece_id = piece.id;
        
        // 更新棋子位置
        piece.position = to;
        
        // 检查吃子
        let captured = crate::game::rules::calculate_captures(self, piece_id);
        
        // 收集被吃棋子的记录（包含位置信息）
        let mut captured_records = Vec::new();
        for &captured_id in &captured {
            if let Some(p) = self.piece_by_id(captured_id) {
                captured_records.push(CapturedRecord {
                    piece_id: captured_id,
                    position: p.position, // 记录被吃时的位置
                });
            }
            if let Some(p) = self.piece_by_id_mut(captured_id) {
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

    /// 悔棋（撤销移动）
    pub fn undo_move(&mut self, record: &MoveRecord) -> Result<()> {
        // 恢复移动的棋子位置
        if let Some(piece) = self.piece_by_id_mut(record.piece_id) {
            piece.position = record.from;
            piece.active = true;
        }

        // 恢复被吃的棋子（包括位置）
        for captured_record in &record.captured {
            if let Some(piece) = self.piece_by_id_mut(captured_record.piece_id) {
                piece.position = captured_record.position; // 恢复被吃时的位置
                piece.active = true;
            }
        }

        Ok(())
    }

    /// 获取某位置在屏幕上的坐标（用于渲染）
    /// 
    /// 棋子放在交叉点上（线的交点），而不是格子中间
    /// 
    /// 参数:
    /// - board_rect: 棋盘在屏幕上的矩形区域 (x, y, width, height)
    /// - pos: 棋盘坐标 (x, y)，范围 0-3
    /// 
    /// 返回: 屏幕坐标 (x, y)
    pub fn board_to_screen(board_rect: (f32, f32, f32, f32), pos: (u8, u8)) -> (f32, f32) {
        let (bx, by, bw, bh) = board_rect;
        // 3x3格子，4x4交叉点，格子大小为 width / 3
        let cell_w = bw / (BOARD_SIZE - 1) as f32;
        let cell_h = bh / (BOARD_SIZE - 1) as f32;
        
        // (0,0) 在左下角，棋子放在交叉点上
        let screen_x = bx + pos.0 as f32 * cell_w;
        let screen_y = by + bh - pos.1 as f32 * cell_h;
        
        (screen_x, screen_y)
    }

    /// 将屏幕坐标转换为棋盘坐标
    /// 
    /// 棋子放在交叉点上（线的交点）
    /// 
    /// 参数:
    /// - board_rect: 棋盘在屏幕上的矩形区域
    /// - screen_pos: 屏幕坐标
    /// - tolerance: 容错范围（以格子大小的比例表示，如0.3表示30%）
    /// 
    /// 返回: 可选的棋盘坐标
    pub fn screen_to_board(
        board_rect: (f32, f32, f32, f32),
        screen_pos: (f32, f32),
        tolerance: f32,
    ) -> Option<(u8, u8)> {
        let (bx, by, bw, bh) = board_rect;
        // 3x3格子，4x4交叉点，格子大小为 width / 3
        let cell_w = bw / (BOARD_SIZE - 1) as f32;
        let cell_h = bh / (BOARD_SIZE - 1) as f32;

        // 计算相对于棋盘左下角的坐标
        let rel_x = screen_pos.0 - bx;
        let rel_y = bh - (screen_pos.1 - by); // 翻转Y轴

        // 计算最近的交叉点索引（0-3）
        let board_x = (rel_x / cell_w).round() as i32;
        let board_y = (rel_y / cell_h).round() as i32;

        // 检查是否在容错范围内（以交叉点为中心）
        let cross_x = board_x as f32 * cell_w;
        let cross_y = board_y as f32 * cell_h;
        
        let dist_x = (rel_x - cross_x).abs();
        let dist_y = (rel_y - cross_y).abs();
        
        let max_dist = cell_w.min(cell_h) * tolerance;

        if dist_x <= max_dist && dist_y <= max_dist {
            if Self::is_valid_pos(board_x as i8, board_y as i8) {
                return Some((board_x as u8, board_y as u8));
            }
        }

        None
    }
}
