//! 棋盘视图渲染

use egui::{Color32, Pos2, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2, Image, TextureHandle, Context};

use crate::game::board::BOARD_SIZE;
use crate::game::piece::{Piece, Side};
use std::sync::Arc;

/// 棋子图片资源（96x96 像素，按100%原大小显示）
const BLACK_STONE_PNG: &[u8] = include_bytes!("../assets/images/black_stone.png");
const WHITE_STONE_PNG: &[u8] = include_bytes!("../assets/images/white_stone.png");

/// 棋子图片尺寸
const STONE_SIZE: f32 = 96.0;

/// 棋盘视图
#[derive(Clone)]
pub struct BoardView {
    /// 棋盘矩形区域（屏幕坐标）
    pub rect: Rect,
    /// 格子大小
    pub cell_size: f32,
    /// 棋子半径（用于点击检测）
    pub piece_radius: f32,
    /// 是否翻转棋盘（玩家执白时翻转，使白棋在下方）
    pub flip: bool,
    /// 黑子纹理
    black_stone: Option<Arc<TextureHandle>>,
    /// 白子纹理
    white_stone: Option<Arc<TextureHandle>>,
}

impl BoardView {
    /// 创建棋盘视图
    /// 
    /// # Arguments
    /// * `center` - 棋盘中心点
    /// * `size` - 棋盘大小
    /// * `flip` - 是否翻转棋盘（玩家执白时为true，使玩家棋子在下方）
    /// * `ctx` - egui 上下文，用于加载纹理
    pub fn new(center: Pos2, size: f32, flip: bool, ctx: &Context) -> Self {
        let _half = size / 2.0;
        let rect = Rect::from_center_size(center, Vec2::new(size, size));
        // 3x3格子，4x4交叉点，格子大小为 size / 3
        let cell_size = size / (BOARD_SIZE - 1) as f32;
        // 棋子点击检测半径使用图片尺寸的一半
        let piece_radius = STONE_SIZE / 2.0;

        // 加载棋子图片纹理
        let black_stone = Self::load_stone_texture(ctx, BLACK_STONE_PNG, "black_stone");
        let white_stone = Self::load_stone_texture(ctx, WHITE_STONE_PNG, "white_stone");

        Self {
            rect,
            cell_size,
            piece_radius,
            flip,
            black_stone,
            white_stone,
        }
    }

    /// 加载棋子图片纹理
    fn load_stone_texture(ctx: &Context, bytes: &[u8], name: &str) -> Option<Arc<TextureHandle>> {
        // 使用 image 库解码 PNG
        match image::load_from_memory(bytes) {
            Ok(image) => {
                let image = image.to_rgba8();
                let size = [image.width() as usize, image.height() as usize];
                let pixels = image.as_raw();
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels);
                let texture = ctx.load_texture(name, color_image, egui::TextureOptions::default());
                Some(Arc::new(texture))
            }
            Err(e) => {
                eprintln!("Failed to load stone texture '{}': {}", name, e);
                None
            }
        }
    }

    /// 渲染棋盘背景
    pub fn draw_board(&self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(self.rect, Sense::click_and_drag());

        let painter = ui.painter();

        // 绘制棋盘背景
        painter.rect_filled(self.rect, Rounding::ZERO, Color32::from_rgb(240, 217, 181));

        // 绘制网格线
        // 3x3格子 = 4条线，棋子放在4x4交叉点上
        let stroke = Stroke::new(2.0, Color32::from_rgb(101, 67, 33));
        
        // 横线 (4条，i=0,1,2,3)
        for i in 0..BOARD_SIZE {
            let y = self.rect.min.y + i as f32 * self.cell_size;
            painter.line_segment(
                [Pos2::new(self.rect.min.x, y), Pos2::new(self.rect.max.x, y)],
                stroke,
            );
        }
        
        // 纵线 (4条，i=0,1,2,3)
        for i in 0..BOARD_SIZE {
            let x = self.rect.min.x + i as f32 * self.cell_size;
            painter.line_segment(
                [Pos2::new(x, self.rect.min.y), Pos2::new(x, self.rect.max.y)],
                stroke,
            );
        }

        response
    }

    /// 渲染单个棋子（使用图片，100%原大小显示）
    pub fn draw_piece(&self, ui: &mut Ui, piece: &Piece, is_dragging: bool, drag_pos: Option<Pos2>) {
        let pos = if is_dragging {
            drag_pos.unwrap_or_else(|| self.board_to_screen(piece.position))
        } else {
            self.board_to_screen(piece.position)
        };

        // 获取对应的棋子纹理
        let texture = match piece.side {
            Side::Black => self.black_stone.as_ref(),
            Side::White => self.white_stone.as_ref(),
        };

        if let Some(texture) = texture {
            // 图片按100%原大小显示，居中于交叉点
            let image_size = Vec2::new(STONE_SIZE, STONE_SIZE);
            let image_rect = Rect::from_center_size(pos, image_size);
            
            let image = Image::from_texture(texture.as_ref())
                .fit_to_exact_size(image_size);
            
            ui.put(image_rect, image);
        } else {
            // 如果图片加载失败，回退到代码绘制
            let painter = ui.painter();
            let color = match piece.side {
                Side::Black => Color32::from_rgb(30, 30, 30),
                Side::White => Color32::from_rgb(240, 240, 240),
            };
            painter.circle_filled(pos, self.piece_radius, color);
        }
    }

    /// 将棋盘坐标转换为屏幕坐标
    /// 
    /// 棋子放在交叉点上（线的交点），而不是格子中间
    /// 如果 flip 为 true，则翻转棋盘，使白棋在下方
    pub fn board_to_screen(&self, pos: (u8, u8)) -> Pos2 {
        let (bx, by) = if self.flip {
            // 翻转：x镜像，y镜像
            (BOARD_SIZE as u8 - 1 - pos.0, BOARD_SIZE as u8 - 1 - pos.1)
        } else {
            // 正常：黑棋在下方
            pos
        };
        
        // 棋子放在交叉点上（线的交点），不需要加 cell_size/2.0 偏移
        let x = self.rect.min.x + bx as f32 * self.cell_size;
        let y = self.rect.max.y - by as f32 * self.cell_size;
        Pos2::new(x, y)
    }

    /// 将屏幕坐标转换为棋盘坐标（带容错）
    /// 
    /// 棋子放在交叉点上（线的交点）
    /// 如果 flip 为 true，则翻转棋盘坐标
    pub fn screen_to_board(&self, pos: Pos2, tolerance: f32) -> Option<(u8, u8)> {
        let rel_x = pos.x - self.rect.min.x;
        let rel_y = self.rect.max.y - pos.y;

        // 计算最近的交叉点索引（0-3）
        let board_x = (rel_x / self.cell_size).round() as i32;
        let board_y = (rel_y / self.cell_size).round() as i32;

        // 检查是否在容错范围内（以交叉点为中心）
        let cross_x = board_x as f32 * self.cell_size;
        let cross_y = board_y as f32 * self.cell_size;

        let dist_x = (rel_x - cross_x).abs();
        let dist_y = (rel_y - cross_y).abs();
        let max_dist = self.cell_size * tolerance;

        if dist_x <= max_dist && dist_y <= max_dist {
            if board_x >= 0 && board_x < BOARD_SIZE as i32 
                && board_y >= 0 && board_y < BOARD_SIZE as i32 {
                let (bx, by) = (board_x as u8, board_y as u8);
                // 如果翻转，需要转换回原始棋盘坐标
                if self.flip {
                    Some((BOARD_SIZE as u8 - 1 - bx, BOARD_SIZE as u8 - 1 - by))
                } else {
                    Some((bx, by))
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 检查点是否在棋子内
    pub fn hit_test_piece(&self, pos: Pos2, piece_pos: (u8, u8)) -> bool {
        let piece_screen_pos = self.board_to_screen(piece_pos);
        let dist = (pos - piece_screen_pos).length();
        dist <= self.piece_radius
    }

    /// 绘制拖拽中的棋子（半透明）
    pub fn draw_dragging_piece(&self, ui: &mut Ui, piece: &Piece, mouse_pos: Pos2) {
        let painter = ui.painter();

        let color = match piece.side {
            Side::Black => Color32::from_rgba_premultiplied(30, 30, 30, 180),
            Side::White => Color32::from_rgba_premultiplied(240, 240, 240, 180),
        };

        // 绘制半透明棋子跟随鼠标
        painter.circle_filled(mouse_pos, self.piece_radius, color);
        
        // 绘制原位置虚线提示
        let original_pos = self.board_to_screen(piece.position);
        painter.circle_stroke(
            original_pos,
            self.piece_radius,
            Stroke::new(2.0, Color32::from_rgba_premultiplied(128, 128, 128, 128)),
        );
    }

    /// 绘制动画中的棋子
    pub fn draw_animated_piece(&self, ui: &mut Ui, piece: &Piece, current_pos: Pos2) {
        let painter = ui.painter();

        let color = match piece.side {
            Side::Black => Color32::from_rgb(30, 30, 30),
            Side::White => Color32::from_rgb(240, 240, 240),
        };

        painter.circle_filled(current_pos, self.piece_radius, color);
    }

    /// 绘制被吃棋子动画（缩小淡出）
    pub fn draw_capturing_piece(&self, ui: &mut Ui, piece: &Piece, progress: f32) {
        let painter = ui.painter();

        let alpha = ((1.0 - progress) * 255.0) as u8;
        let radius = self.piece_radius * (1.0 - progress);

        let color = match piece.side {
            Side::Black => Color32::from_rgba_premultiplied(30, 30, 30, alpha),
            Side::White => Color32::from_rgba_premultiplied(240, 240, 240, alpha),
        };

        let pos = self.board_to_screen(piece.position);
        painter.circle_filled(pos, radius, color);
    }

    /// 绘制带透明度的棋子（用于悔棋动画渐显效果）
    pub fn draw_piece_with_alpha(&self, ui: &mut Ui, piece: &Piece, pos: Pos2, alpha: u8) {
        let painter = ui.painter();

        let color = match piece.side {
            Side::Black => Color32::from_rgba_premultiplied(30, 30, 30, alpha),
            Side::White => Color32::from_rgba_premultiplied(240, 240, 240, alpha),
        };

        let stroke_color = if alpha > 100 {
            match piece.side {
                Side::Black => Color32::from_rgba_premultiplied(80, 80, 80, alpha),
                Side::White => Color32::from_rgba_premultiplied(180, 180, 180, alpha),
            }
        } else {
            Color32::TRANSPARENT
        };

        // 绘制棋子本体
        painter.circle_filled(pos, self.piece_radius, color);
        
        // 绘制边框（当透明度足够时）
        if alpha > 50 {
            painter.circle_stroke(pos, self.piece_radius, Stroke::new(2.0, stroke_color));
        }
    }

    /// 绘制可落子提示
    pub fn draw_valid_move_hint(&self, ui: &mut Ui, pos: (u8, u8)) {
        let painter = ui.painter();
        let screen_pos = self.board_to_screen(pos);
        let radius = self.piece_radius * 0.3;

        painter.circle_filled(
            screen_pos,
            radius,
            Color32::from_rgba_premultiplied(100, 200, 100, 150),
        );
    }

    /// 绘制原始位置标记（棋子被吸附时显示）
    pub fn draw_origin_marker(&self, ui: &mut Ui, pos: (u8, u8)) {
        let painter = ui.painter();
        let screen_pos = self.board_to_screen(pos);
        
        // 绘制虚线圆圈表示原始位置
        let stroke = Stroke::new(
            2.0,
            Color32::from_rgba_premultiplied(100, 100, 100, 150),
        );
        
        // 绘制虚线圆
        let segments = 12;
        for i in 0..segments {
            if i % 2 == 0 {
                let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                
                let p1 = screen_pos + Vec2::new(angle1.cos(), angle1.sin()) * self.piece_radius * 1.1;
                let p2 = screen_pos + Vec2::new(angle2.cos(), angle2.sin()) * self.piece_radius * 1.1;
                
                painter.line_segment([p1, p2], stroke);
            }
        }
        
        // 绘制中心小点
        painter.circle_filled(
            screen_pos,
            self.piece_radius * 0.15,
            Color32::from_rgba_premultiplied(150, 150, 150, 200),
        );
    }
}
