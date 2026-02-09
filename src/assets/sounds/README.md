# 音效文件说明

此目录包含游戏所需的音效文件。所有音效均来自免费可商用资源。

## 文件列表

| 文件名 | 用途 | 来源 |
|--------|------|------|
| `click.wav` | 点击棋子 | Kenney UI Audio (CC0) |
| `place.wav` | 合法落子 | OpenGameArt - Various SFX (CC0) |
| `invalid.wav` | 非法落子 | Kenney UI Audio (CC0) |
| `capture.wav` | 吃子/担子 | Kenney UI Audio (CC0) |
| `win.wav` | 玩家获胜 | OpenGameArt - Victory (CC0) |
| `lose.wav` | 电脑获胜 | OpenGameArt - Game Over (CC0) |
| `draw.wav` | 平局 | OpenGameArt - Menu Select (CC0) |

## 音效来源详情

### 1. Kenney UI Audio (CC0)
- 网址: https://kenney.nl/assets/ui-audio
- 包含 50 个高质量 UI 音效
- 用于: click, invalid, capture

### 2. OpenGameArt (CC0)
- Victory.wav: https://opengameart.org/content/victory
- GameOver.wav: https://opengameart.org/content/game-over-soundold-school
- Various SFX: https://opengameart.org/content/various-sound-effects-0
- 用于: win, lose, place, draw

## 授权说明

所有音效均采用 **CC0 (Creative Commons Zero)** 协议，可自由用于个人和商业项目，无需署名。

## 格式规格

- **格式**: WAV
- **采样率**: 44100 Hz (部分 96000 Hz)
- **声道**: 单声道或立体声
- **位深**: 16-bit

## 原始备份

原始示例音效已备份至 `backup_original/` 目录。

## 替换方法

如需替换音效：
1. 下载新的音效文件（WAV 格式）
2. 将文件重命名为对应名称（如 `click.wav`）
3. 替换此目录下的同名文件
4. 重新编译程序：`cargo build --release`

注意：程序使用 `include_bytes!` 宏将音效文件嵌入到可执行文件中，文件大小会影响最终程序体积。
