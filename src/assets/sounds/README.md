# 音效文件说明

此目录包含游戏所需的音效文件。当前文件是使用 Python 生成的占位符音效，建议替换为真实的高质量音效。

## 文件列表

| 文件名 | 用途 | 规格建议 |
|--------|------|---------|
| `click.wav` | 点击棋子 | 短促清脆，约100ms |
| `place.wav` | 合法落子 | 木质或石质碰撞感，约200ms |
| `invalid.wav` | 非法落子 | 低沉错误提示，约300ms |
| `capture.wav` | 吃子/担子 | 略长，有"吃掉"的感觉，约400ms |
| `win.wav` | 玩家获胜 | 欢快胜利音效，约800ms |
| `lose.wav` | 电脑获胜 | 低沉失败音效，约600ms |
| `draw.wav` | 平局 | 中性音效，约500ms |

## 推荐音效来源

### 免费可商用资源

1. **Kenney UI Audio** (CC0)
   - 网址: https://kenney.nl/assets/ui-audio
   - 包含 50 个 UI 音效

2. **Pixabay** (免版税)
   - 搜索: https://pixabay.com/sound-effects/
   - 搜索关键词: `wood click`, `stone drop`, `success`, `error`

3. **Freesound** (CC-BY/CC0)
   - 网址: https://freesound.org/
   - 搜索关键词: `chess piece`, `wooden click`, `game piece`

### 具体推荐下载

| 音效类型 | Pixabay 搜索链接 |
|---------|-----------------|
| 点击 | https://pixabay.com/sound-effects/search/click/ |
| 木质 | https://pixabay.com/sound-effects/search/wood/ |
| 石头掉落 | https://pixabay.com/sound-effects/search/stone%20drop/ |
| 成功 | https://pixabay.com/sound-effects/search/success/ |
| 失败 | https://pixabay.com/sound-effects/search/failure/ |

## 替换方法

1. 从上述网站下载合适的音效文件（WAV 或 MP3 格式）
2. 将文件重命名为对应名称（如 `click.wav`）
3. 替换此目录下的同名文件
4. 重新编译程序：`cargo build --release`

## 格式要求

- **格式**: WAV (推荐) 或 MP3
- **采样率**: 44100 Hz
- **声道**: 单声道或立体声
- **位深**: 16-bit

注意：程序使用 `include_bytes!` 宏将音效文件嵌入到可执行文件中，因此文件大小会影响最终程序体积。建议使用适当压缩的音效文件。
