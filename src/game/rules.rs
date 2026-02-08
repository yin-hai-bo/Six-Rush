//! 游戏规则验证

use crate::game::board::Board;
use crate::game::piece::Side;
use crate::game::GameResult;

/// 检查移动是否合法
/// 
/// 参数:
/// - board: 当前棋盘
/// - from: 起始位置
/// - to: 目标位置
/// - side: 执子方
/// 
/// 返回: 是否合法
pub fn is_valid_move(board: &Board, from: (u8, u8), to: (u8, u8), side: Side) -> bool {
    // 检查起始位置有己方棋子
    let piece = match board.piece_at(from.0, from.1) {
        Some(p) if p.side == side => p,
        _ => return false,
    };

    // 检查目标位置为空且在棋盘内
    if !Board::is_valid_pos(to.0 as i8, to.1 as i8) {
        return false;
    }
    if !board.is_empty(to.0, to.1) {
        return false;
    }

    // 检查移动距离（只能上下左右移动一格）
    let dx = (to.0 as i8) - (from.0 as i8);
    let dy = (to.1 as i8) - (from.1 as i8);

    // 只允许水平或垂直移动一格
    let is_horizontal = dy == 0 && dx.abs() == 1;
    let is_vertical = dx == 0 && dy.abs() == 1;

    if !is_horizontal && !is_vertical {
        return false;
    }

    // 棋子状态检查（不能移动被吃掉的棋子）
    if !piece.active {
        return false;
    }

    true
}

/// 计算移动后的吃子
/// 
/// 返回: 被吃掉的棋子ID列表
pub fn calculate_captures(board: &Board, moved_piece_id: u8) -> Vec<u8> {
    let moved_piece = match board.piece_by_id(moved_piece_id) {
        Some(p) if p.active => p,
        _ => return Vec::new(),
    };

    let side = moved_piece.side;
    let (x, y) = moved_piece.position;
    let mut captured = Vec::new();

    if board.is_single_piece_mode() {
        // 单子状态：检查"担"吃
        // 检查水平方向
        check_single_piece_capture(board, x, y, side, true, &mut captured);
        // 检查垂直方向
        check_single_piece_capture(board, x, y, side, false, &mut captured);
    } else {
        // 正常状态：检查"二比一"吃棋
        // 水平方向
        check_two_vs_one(board, x, y, side, true, moved_piece_id, &mut captured);
        // 垂直方向
        check_two_vs_one(board, x, y, side, false, moved_piece_id, &mut captured);
    }

    captured
}

/// 检查"二比一"吃棋（严格规则）
/// 
/// 必须满足：
/// 1. 这一行/列上有且只有3枚棋子
/// 2. 这3枚棋子紧紧相连（无间隔）
/// 3. 其中两枚是本方棋子（且一枚是刚移动的），一枚是对方棋子
/// 4. 3枚棋子占据的格子两侧必须是边界或空点
fn check_two_vs_one(
    board: &Board,
    x: u8,
    y: u8,
    side: Side,
    horizontal: bool,
    moved_piece_id: u8,
    captured: &mut Vec<u8>,
) {
    let dx = if horizontal { 1 } else { 0 };
    let dy = if horizontal { 0 } else { 1 };

    // 检查两种可能的"二比一"模式：
    // 模式1: [本方][本方(刚移动)][对方] - 本方在左/下，对方在右/上
    // 模式2: [对方][本方(刚移动)][本方] - 对方在左/下，本方在右/上
    // 模式3: [本方(刚移动)][本方][对方] - 刚移动的本方在最左/下
    // 模式4: [对方][本方][本方(刚移动)] - 刚移动的本方在最右/上

    // 先检查从刚移动棋子向左/下的情况
    check_two_vs_one_in_direction(board, x, y, side, dx, dy, moved_piece_id, captured);
    // 再检查从刚移动棋子向右/上的情况
    check_two_vs_one_in_direction(board, x, y, side, -dx, -dy, moved_piece_id, captured);
}

/// 在指定方向检查"二比一"吃棋
fn check_two_vs_one_in_direction(
    board: &Board,
    x: u8,
    y: u8,
    side: Side,
    _dx: i8,
    dy: i8,
    moved_piece_id: u8,
    captured: &mut Vec<u8>,
) {
    // 确认刚移动的棋子仍然存在且活跃
    if board.piece_by_id(moved_piece_id).map_or(true, |p| !p.active) {
        return;
    }

    // 收集这一行/列上所有棋子的位置
    // 水平方向：固定y，变化x；垂直方向：固定x，变化y
    let is_horizontal = dy == 0;

    // 获取这一行/列上的所有棋子（按位置排序）
    let pieces_on_line: Vec<_> = board.pieces.iter()
        .filter(|p| p.active && if is_horizontal { p.position.1 == y } else { p.position.0 == x })
        .map(|p| p)
        .collect();

    // 必须有且只有3枚棋子
    if pieces_on_line.len() != 3 {
        return;
    }

    // 检查这3枚棋子是否紧紧相连（相邻位置差为1）
    let mut positions: Vec<(u8, u8)> = pieces_on_line.iter().map(|p| p.position).collect();
    positions.sort_by(|a, b| {
        let a_coord = if is_horizontal { a.0 } else { a.1 };
        let b_coord = if is_horizontal { b.0 } else { b.1 };
        a_coord.cmp(&b_coord)
    });

    // 检查是否紧紧相连
    for i in 0..positions.len() - 1 {
        let coord1 = if is_horizontal { positions[i].0 } else { positions[i].1 };
        let coord2 = if is_horizontal { positions[i + 1].0 } else { positions[i + 1].1 };
        if coord2 - coord1 != 1 {
            return; // 不相连
        }
    }

    // 检查两侧是否为空或边界
    let first_coord = if is_horizontal { positions[0].0 } else { positions[0].1 };
    let last_coord = if is_horizontal { positions[2].0 } else { positions[2].1 };

    // 检查左侧/下方
    let left_coord = first_coord as i8 - 1;
    if left_coord >= 0 {
        let check_pos = if is_horizontal { (left_coord as u8, y) } else { (x, left_coord as u8) };
        if !board.is_empty(check_pos.0, check_pos.1) {
            return; // 左侧/下方有棋子
        }
    }

    // 检查右侧/上方
    let right_coord = last_coord as i8 + 1;
    if right_coord < 4 {
        let check_pos = if is_horizontal { (right_coord as u8, y) } else { (x, right_coord as u8) };
        if !board.is_empty(check_pos.0, check_pos.1) {
            return; // 右侧/上方有棋子
        }
    }

    // 现在确定是3枚棋子紧紧相连，检查是否满足"二比一"条件
    // 
    // 有效排列必须是以下两种之一：
    // 1. [本方][本方][对方] - 本方在位置0-1（相邻），对方在位置2
    // 2. [对方][本方][本方] - 对方在位置0，本方在位置1-2（相邻）
    //
    // 无效排列（不能吃子）：
    // [本方][对方][本方] - 本方不相邻（中间隔着对方）

    // 获取3枚棋子按位置排序后的信息
    let sorted_pieces: Vec<_> = pieces_on_line.iter()
        .map(|p| (p.position, p.side, p.id))
        .collect();

    // 确定每枚棋子的位置索引（0, 1, 2）
    let mut pieces_with_index: Vec<(usize, Side, u8)> = Vec::new();
    for (pos, side_val, id) in sorted_pieces {
        let idx = if is_horizontal {
            (pos.0 - first_coord) as usize
        } else {
            (pos.1 - first_coord) as usize
        };
        pieces_with_index.push((idx, side_val, id));
    }

    // 检查本方两枚棋子是否相邻
    let own_pieces_indices: Vec<usize> = pieces_with_index.iter()
        .filter(|(_, s, _)| *s == side)
        .map(|(idx, _, _)| *idx)
        .collect();
    let enemy_pieces_indices: Vec<usize> = pieces_with_index.iter()
        .filter(|(_, s, _)| *s != side)
        .map(|(idx, _, _)| *idx)
        .collect();

    // 必须是2枚本方，1枚对方
    if own_pieces_indices.len() != 2 || enemy_pieces_indices.len() != 1 {
        return;
    }

    // 本方两枚棋子必须相邻（索引差为1）
    let own_idx_diff = (own_pieces_indices[0] as i8 - own_pieces_indices[1] as i8).abs();
    if own_idx_diff != 1 {
        return; // 本方棋子不相邻，不能吃子
    }

    // 本方棋子中必须有刚移动的那枚
    let own_piece_ids: Vec<u8> = pieces_with_index.iter()
        .filter(|(_, s, _)| *s == side)
        .map(|(_, _, id)| *id)
        .collect();
    if !own_piece_ids.contains(&moved_piece_id) {
        return;
    }

    // 所有条件满足，吃掉对方棋子
    let enemy_id = pieces_with_index.iter()
        .find(|(_, s, _)| *s != side)
        .map(|(_, _, id)| *id)
        .unwrap();
    
    if !captured.contains(&enemy_id) {
        captured.push(enemy_id);
    }
}

/// 检查单子"担"吃
fn check_single_piece_capture(
    board: &Board,
    x: u8,
    y: u8,
    side: Side,
    horizontal: bool,
    captured: &mut Vec<u8>,
) {
    let dx = if horizontal { 1 } else { 0 };
    let dy = if horizontal { 0 } else { 1 };

    // 检查正方向
    let nx = x as i8 + dx;
    let ny = y as i8 + dy;

    // 检查反方向
    let rx = x as i8 - dx;
    let ry = y as i8 - dy;

    if !Board::is_valid_pos(nx, ny) || !Board::is_valid_pos(rx, ry) {
        return;
    }

    // 检查是否两侧都是对方棋子
    if let Some(p1) = board.piece_at(nx as u8, ny as u8) {
        if let Some(p2) = board.piece_at(rx as u8, ry as u8) {
            if p1.side != side && p2.side != side && p1.active && p2.active {
                // 形成 "对方-单子-对方"，担吃两枚对方棋子
                if !captured.contains(&p1.id) {
                    captured.push(p1.id);
                }
                if !captured.contains(&p2.id) {
                    captured.push(p2.id);
                }
            }
        }
    }
}

/// 检查游戏是否结束
/// 
/// 参数:
/// - board: 当前棋盘
/// - side_to_move: 轮到行棋的一方
/// - player_side: 玩家执哪一方
/// 
/// 返回: 如果有结果则返回 GameResult
pub fn check_game_end(board: &Board, side_to_move: Side, player_side: Side) -> Option<GameResult> {
    let black_count = board.count_active(Side::Black);
    let white_count = board.count_active(Side::White);

    // 检查无子判负
    if black_count == 0 {
        // 黑方无子，白方胜
        return Some(if player_side == Side::White {
            GameResult::PlayerWin
        } else {
            GameResult::AiWin
        });
    }
    if white_count == 0 {
        // 白方无子，黑方胜
        return Some(if player_side == Side::Black {
            GameResult::PlayerWin
        } else {
            GameResult::AiWin
        });
    }

    // 检查平局：双方均不超过2枚棋子
    if black_count <= 2 && white_count <= 2 {
        return Some(GameResult::Draw);
    }

    // 检查困毙
    if is_stalemated(board, side_to_move) {
        // 轮到 side_to_move 行棋，但无法移动，side_to_move判负
        // side_to_move的对手获胜
        let winner = side_to_move.opposite();
        return Some(if winner == player_side {
            GameResult::PlayerWin
        } else {
            GameResult::AiWin
        });
    }

    None
}

/// 检查某方是否被困毙（无合法移动）
pub fn is_stalemated(board: &Board, side: Side) -> bool {
    let pieces: Vec<_> = board.active_pieces_of(side).iter().map(|&p| p.clone()).collect();

    for piece in pieces {
        let (x, y) = piece.position;

        // 检查四个方向
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        for (dx, dy) in directions {
            let nx = x as i8 + dx;
            let ny = y as i8 + dy;

            if Board::is_valid_pos(nx, ny) && board.is_empty(nx as u8, ny as u8) {
                return false; // 至少有一个合法移动
            }
        }
    }

    true // 无合法移动，困毙
}

/// 获取某方所有合法移动
pub fn get_valid_moves(board: &Board, side: Side) -> Vec<((u8, u8), (u8, u8))> {
    let mut moves = Vec::new();
    let pieces: Vec<_> = board.active_pieces_of(side).iter().map(|&p| p.clone()).collect();

    for piece in pieces {
        let (x, y) = piece.position;
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in directions {
            let nx = x as i8 + dx;
            let ny = y as i8 + dy;

            if Board::is_valid_pos(nx, ny) && board.is_empty(nx as u8, ny as u8) {
                moves.push(((x, y), (nx as u8, ny as u8)));
            }
        }
    }

    moves
}
