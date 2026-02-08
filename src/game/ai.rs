//! AI算法实现

use crate::game::board::Board;
use crate::game::piece::Side;
use crate::game::rules::{get_valid_moves, is_stalemated};
use anyhow::Result;

/// AI玩家
pub struct AiPlayer {
    level: u8,
}

impl AiPlayer {
    /// 创建AI玩家
    pub fn new(level: u8) -> Self {
        Self { level: level.clamp(1, 5) }
    }

    /// 选择走法
    pub fn select_move(&self, board: &Board, side: Side) -> Result<((u8, u8), (u8, u8))> {
        let valid_moves = get_valid_moves(board, side);
        
        if valid_moves.is_empty() {
            return Err(anyhow::anyhow!("无合法移动"));
        }

        match self.level {
            1 => Self::random_move(&valid_moves),
            2 => self.simple_eval_move(board, &valid_moves, side),
            3 => self.minimax_move(board, &valid_moves, side, 4),
            4 => self.minimax_move(board, &valid_moves, side, 6),
            5 => self.optimal_move(board, &valid_moves, side),
            _ => Self::random_move(&valid_moves),
        }
    }

    /// Level 1: 完全随机
    fn random_move(moves: &[((u8, u8), (u8, u8))]) -> Result<((u8, u8), (u8, u8))> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..moves.len());
        moves.get(idx).copied()
            .ok_or_else(|| anyhow::anyhow!("无可用移动"))
    }

    /// Level 2: 带简单评估的随机
    fn simple_eval_move(
        &self,
        board: &Board,
        moves: &[((u8, u8), (u8, u8))],
        _side: Side,
    ) -> Result<((u8, u8), (u8, u8))> {
        // 优先选择能吃子的走法
        let capturing_moves: Vec<_> = moves
            .iter()
            .filter(|(from, to)| {
                // 模拟移动并检查是否能吃子
                let mut test_board = board.clone();
                if let Ok(record) = test_board.execute_move(*from, *to, _side) {
                    !record.captured.is_empty()
                } else {
                    false
                }
            })
            .copied()
            .collect();

        if !capturing_moves.is_empty() {
            Self::random_move(&capturing_moves)
        } else {
            Self::random_move(moves)
        }
    }

    /// Level 3-4: Minimax算法
    fn minimax_move(
        &self,
        board: &Board,
        moves: &[((u8, u8), (u8, u8))],
        side: Side,
        depth: i32,
    ) -> Result<((u8, u8), (u8, u8))> {
        let mut best_move = None;
        let mut best_score = i32::MIN;

        for (from, to) in moves.iter().copied() {
            let mut test_board = board.clone();
            if test_board.execute_move(from, to, side).is_ok() {
                let score = self.minimax(&test_board, depth - 1, false, side, i32::MIN, i32::MAX);
                if score > best_score {
                    best_score = score;
                    best_move = Some((from, to));
                }
            }
        }

        best_move.ok_or_else(|| anyhow::anyhow!("无法找到最佳移动"))
    }

    /// Minimax算法（带Alpha-Beta剪枝）
    fn minimax(
        &self,
        board: &Board,
        depth: i32,
        is_maximizing: bool,
        ai_side: Side,
        mut alpha: i32,
        mut beta: i32,
    ) -> i32 {
        if depth == 0 {
            return self.evaluate(board, ai_side);
        }

        let current_side = if is_maximizing { ai_side } else { ai_side.opposite() };
        let moves = get_valid_moves(board, current_side);

        if moves.is_empty() {
            // 无合法移动，困毙
            return if is_maximizing { i32::MIN + 100 } else { i32::MAX - 100 };
        }

        if is_maximizing {
            let mut max_eval = i32::MIN;
            for (from, to) in moves {
                let mut test_board = board.clone();
                if test_board.execute_move(from, to, current_side).is_ok() {
                    let eval = self.minimax(&test_board, depth - 1, false, ai_side, alpha, beta);
                    max_eval = max_eval.max(eval);
                    alpha = alpha.max(eval);
                    if beta <= alpha {
                        break; // Beta剪枝
                    }
                }
            }
            max_eval
        } else {
            let mut min_eval = i32::MAX;
            for (from, to) in moves {
                let mut test_board = board.clone();
                if test_board.execute_move(from, to, ai_side).is_ok() {
                    let eval = self.minimax(&test_board, depth - 1, true, ai_side, alpha, beta);
                    min_eval = min_eval.min(eval);
                    beta = beta.min(eval);
                    if beta <= alpha {
                        break; // Alpha剪枝
                    }
                }
            }
            min_eval
        }
    }

    /// 评估函数
    fn evaluate(&self, board: &Board, ai_side: Side) -> i32 {
        let player_side = ai_side.opposite();
        let ai_count = board.count_active(ai_side) as i32;
        let player_count = board.count_active(player_side) as i32;

        // 基础评估：棋子数差值 * 100
        let mut score = (ai_count - player_count) * 100;

        // 灵活性评估：可移动方向数
        let ai_moves = get_valid_moves(board, ai_side).len() as i32;
        let player_moves = get_valid_moves(board, player_side).len() as i32;
        score += (ai_moves - player_moves) * 5;

        // 困毙评估 - 这是最重要的
        if is_stalemated(board, player_side) {
            // 玩家被困毙，AI大胜
            score += 10000;
        }
        if is_stalemated(board, ai_side) {
            // AI被困毙，AI大败
            score -= 10000;
        }

        // 单子状态特殊评估
        if player_count == 1 && ai_count >= 2 {
            // 玩家是单子，AI有优势，应该尝试困毙
            // 找到玩家的单子
            if let Some(single_piece) = board.active_pieces_of(player_side).first() {
                let (px, py) = single_piece.position;
                
                // 计算单子周围的空格数（移动空间）
                let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
                let empty_neighbors = directions.iter().filter(|&&(dx, dy)| {
                    let nx = px as i8 + dx;
                    let ny = py as i8 + dy;
                    Board::is_valid_pos(nx, ny) && board.is_empty(nx as u8, ny as u8)
                }).count();
                
                // 单子的移动空间越小，对AI越有利
                score += (4 - empty_neighbors as i32) * 50;
                
                // 鼓励AI棋子靠近单子（围堵）
                for ai_piece in board.active_pieces_of(ai_side) {
                    let dist = ((ai_piece.position.0 as i32 - px as i32).abs()
                        + (ai_piece.position.1 as i32 - py as i32).abs()) as i32;
                    score += (6 - dist) * 10; // 距离越近分数越高
                }
            }
        }

        if ai_count == 1 && player_count >= 2 {
            // AI是单子，处于劣势
            score -= 200;
        }

        score
    }

    /// Level 5: 最优解（完整搜索）
    fn optimal_move(
        &self,
        board: &Board,
        moves: &[((u8, u8), (u8, u8))],
        side: Side,
    ) -> Result<((u8, u8), (u8, u8))> {
        // 对于4x4棋盘和最多12枚棋子，游戏复杂度相对较低
        // 可以尝试完整搜索或使用较深的Minimax
        self.minimax_move(board, moves, side, 8)
    }
}
