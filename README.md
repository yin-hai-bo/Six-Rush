# 六子冲 (Six-Rush)

一款传统棋类的人机对弈游戏，使用 Rust + egui 开发。

> 🚀 本项目由 **Kimi Code** 智能助手完成。

## 游戏规则

详见 [doc/rules.md](doc/rules.md)

## 程序规格

详见 [doc/specification.md](doc/specification.md)

## 游戏状态流转

详见 [doc/states.md](doc/states.md)

## 运行项目

```bash
# 克隆项目
git clone <repository-url>
cd six-rush

# 运行
cargo run --release

# 开发模式（带调试信息）
cargo run
```

## 项目结构

```
src/
├── main.rs          # 程序入口
├── lib.rs           # 库入口
├── game/            # 游戏核心逻辑
│   ├── mod.rs       # 游戏主逻辑与状态机
│   ├── board.rs     # 棋盘定义与坐标转换
│   ├── piece.rs     # 棋子定义与初始布局
│   ├── rules.rs     # 行棋规则与吃子判定
│   ├── state.rs     # 游戏状态定义
│   ├── ai.rs        # AI算法实现（5个难度等级）
│   ├── audio.rs     # 音效系统
│   └── save.rs      # 存档/读档功能
├── ui/              # 用户界面
│   ├── mod.rs       # UI模块入口
│   ├── app.rs       # 主应用与动画控制
│   ├── board_view.rs # 棋盘渲染与交互
│   └── dialogs.rs   # 对话框（新局、游戏结束等）
└── utils/           # 工具函数
    └── mod.rs       # 动画插值与辅助函数
```

## 依赖说明

- **egui/eframe**: 即时模式GUI框架
- **rodio**: 音频播放
- **serde/serde_json**: 序列化（用于存档）
- **chrono**: 时间处理
- **anyhow**: 错误处理
- **rand**: 随机数（用于AI随机走法）
- **rust-i18n**: 国际化支持
- **fontdb**: 字体加载
- **rfd**: 文件对话框
- **image**: 图片处理（棋子PNG）

## License

MIT
