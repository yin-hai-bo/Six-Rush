//! 游戏存档功能

use crate::game::board::{Board, BOARD_SIZE};
use crate::game::piece::{Piece, PieceState, Side};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 存档文件版本
const SAVE_VERSION: u8 = 1;

/// 存档数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    /// 版本号
    version: u8,
    /// 棋子位置数据 [16个位置，每个位置存储棋子信息]
    /// 索引 = y * 4 + x
    /// 值：0=空, 1=黑棋, 2=白棋
    board: [u8; 16],
    /// 当前轮到哪方行棋（加载后默认为玩家回合）
    current_turn: Side,
    /// 玩家执子方
    player_side: Side,
}

/// 保存游戏到文件
pub fn save_game(board: &Board, player_side: Side, path: &Path) -> Result<()> {
    let mut board_data = [0u8; 16];
    
    for piece in &board.pieces {
        if piece.active {
            let (x, y) = piece.position;
            let idx = (y * BOARD_SIZE + x) as usize;
            board_data[idx] = match piece.side {
                Side::Black => 1,
                Side::White => 2,
            };
        }
    }
    
    let save_data = SaveData {
        version: SAVE_VERSION,
        board: board_data,
        current_turn: Side::Black, // 加载后黑方先行
        player_side,
    };
    
    let json = serde_json::to_string_pretty(&save_data)
        .context("序列化存档数据失败")?;
    fs::write(path, json).context("写入存档文件失败")?;
    
    Ok(())
}

/// 从文件加载游戏
pub fn load_game(path: &Path) -> Result<(Board, Side)> {
    let json = fs::read_to_string(path).context("读取存档文件失败")?;
    let save_data: SaveData = serde_json::from_str(&json)
        .context("解析存档数据失败")?;
    
    if save_data.version != SAVE_VERSION {
        anyhow::bail!("不支持的存档版本: {}", save_data.version);
    }
    
    // 重建棋盘
    let mut board = Board::empty();
    let mut piece_id = 1u8;
    
    // 先清空默认棋子
    board.pieces.clear();
    
    for (idx, &cell) in save_data.board.iter().enumerate() {
        if cell != 0 {
            let x = (idx % BOARD_SIZE as usize) as u8;
            let y = (idx / BOARD_SIZE as usize) as u8;
            let side = if cell == 1 { Side::Black } else { Side::White };
            
            board.pieces.push(Piece {
                id: piece_id,
                side,
                position: (x, y),
                state: PieceState::Idle,
                active: true,
            });
            piece_id += 1;
        }
    }
    
    Ok((board, save_data.player_side))
}

/// 检查是否是初始局面
pub fn is_initial_position(board: &Board) -> bool {
    // 初始局面：黑方在下方(y=0,1)，白方在上方(y=2,3)
    // 黑方: (0,0), (1,0), (2,0), (3,0), (0,1), (3,1)
    // 白方: (0,3), (1,3), (2,3), (3,3), (0,2), (3,2)
    
    let expected_black = [(0u8, 0u8), (1, 0), (2, 0), (3, 0), (0, 1), (3, 1)];
    let expected_white = [(0u8, 3u8), (1, 3), (2, 3), (3, 3), (0, 2), (3, 2)];
    
    let active_pieces: Vec<_> = board.pieces.iter().filter(|p| p.active).collect();
    
    if active_pieces.len() != 12 {
        return false;
    }
    
    let black_positions: Vec<_> = active_pieces
        .iter()
        .filter(|p| p.side == Side::Black)
        .map(|p| p.position)
        .collect();
    let white_positions: Vec<_> = active_pieces
        .iter()
        .filter(|p| p.side == Side::White)
        .map(|p| p.position)
        .collect();
    
    black_positions.len() == 6 
        && white_positions.len() == 6
        && expected_black.iter().all(|pos| black_positions.contains(pos))
        && expected_white.iter().all(|pos| white_positions.contains(pos))
}
