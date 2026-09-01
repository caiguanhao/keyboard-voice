# Keyboard Voice

[![release](https://github.com/caiguanhao/keyboard-voice/actions/workflows/release.yml/badge.svg)](https://github.com/caiguanhao/keyboard-voice/actions/workflows/release.yml)

全屏键盘可视化与离线英文语音程序，面向 Raspberry Pi OS 64-bit，也可在 macOS 上以普通窗口测试。

## 本机运行

```bash
cargo run
```

macOS 默认启动为 `1200x675` 窗口。程序只在构建阶段生成语音并把 WAV 嵌入二进制，运行时不加载 TTS 模型、不访问网络，也不依赖系统 TTS。

## 语音后端

支持以下构建后端：

- `piper`：默认后端，使用 Piper 在构建阶段生成 WAV。
- `espeak-ng`：使用 eSpeak-ng 在构建阶段生成 WAV。
- `assets`：直接嵌入开发者提前生成好的 WAV。
- `silent`：生成静音占位 WAV，适合没有语音工具的环境。

默认后端是 `piper`。如果没有显式设置 `PIPER_MODEL`，默认路径是 `models/en_US-lessac-medium.onnx`。构建不会自动联网下载模型：如果检测到 Piper 但模型不存在，构建会显示下载命令并中断；执行该命令后重新运行 Cargo。模型只用于构建阶段，绝不会被打包进最终程序。

### 安装 Piper

官方目前提供 Python 包。macOS 上可以这样安装：

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install piper-tts

mkdir -p models
python -m piper.download_voices \
  --data-dir models \
  en_US-lessac-medium
```

Debian 或 Raspberry Pi OS：

```bash
sudo apt update
sudo apt install -y python3 python3-venv

python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
python -m pip install piper-tts

mkdir -p models
python -m piper.download_voices \
  --data-dir models \
  en_US-lessac-medium
```

下载后应有以下两个文件：

```text
models/en_US-lessac-medium.onnx
models/en_US-lessac-medium.onnx.json
```

`en_US-lessac-medium` 模型本身约 63 MB，只在构建时使用。当前构建脚本会自动检测已激活虚拟环境中的 `python3 -m piper`。官方 CLI 也支持用 `python3 -m piper` 直接生成 WAV。[Piper CLI 文档](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/CLI.md) [voice 文件](https://huggingface.co/rhasspy/piper-voices/tree/main/en/en_US/lessac/medium)

```bash
KEYBOARD_TTS_SOURCE=piper cargo run
```

如果模型已下载到其他位置，再设置 `PIPER_MODEL`；显式路径不存在时构建会显示提示并中断，不会自动替换或下载其他模型：

```bash
KEYBOARD_TTS_SOURCE=piper \
PIPER_MODEL=/path/to/en_US-lessac-medium.onnx \
cargo run
```

如果使用独立 Piper 二进制，而不是 Python 包，再额外设置 `PIPER_BIN`：

```bash
KEYBOARD_TTS_SOURCE=piper \
PIPER_BIN=/path/to/piper \
PIPER_MODEL=models/en_US-lessac-medium.onnx \
cargo run
```

如果开发机完全没有 Piper，且安装了 `espeak-ng`，默认构建会打印警告并回退到 eSpeak-ng。检测到 Piper 但缺少模型时不会回退，而是要求先执行下载命令。也可以显式选择后端：

```bash
# 使用 Piper（默认）
KEYBOARD_TTS_SOURCE=piper PIPER_MODEL=models/en_US-lessac-medium.onnx cargo build --release

# 使用 eSpeak-ng
KEYBOARD_TTS_SOURCE=espeak-ng KEYBOARD_VOICE=en-us+f3 cargo run

# 直接使用预先生成的 WAV，文件名需与音频 ID 对应
KEYBOARD_TTS_SOURCE=assets KEYBOARD_AUDIO_DIR=assets/audio cargo build --release

# 只用于没有语音工具的编译环境
KEYBOARD_TTS_SOURCE=silent cargo build
```

这四种后端只影响构建阶段；最终 Raspberry Pi 程序只包含生成好的 WAV。切换 `KEYBOARD_TTS_SOURCE`、`PIPER_MODEL` 或 `KEYBOARD_VOICE` 后重新运行 Cargo 即可重新生成资源。

生成后的音频缓存位于 `target/debug/keyboard-voice-audio/` 或 `target/release/keyboard-voice-audio/` 下，并按后端和模型指纹分目录。构建脚本再次运行时会复用已有 WAV；只有后端、voice、模型或源 WAV 发生变化时才会重新生成。

构建脚本默认输出音频缓存命中和生成日志。由于 Cargo 默认会隐藏构建脚本的逐文件输出，查看 Piper 的序号、文本和目标 WAV 路径时，请使用 Cargo verbose 模式：

```bash
cargo build -vv
```

如果构建结果没有变化，Cargo 不会重新执行 Piper，也不会输出逐个文件的生成日志。

Piper voice model 的授权协议可能不同，分发前请查看对应的 `MODEL_CARD`。

### eSpeak-ng

macOS 需要先安装：

```bash
brew install espeak-ng
```

使用 eSpeak-ng 时，默认是较柔和的 `en-us+f3` 变体，也可以切换 voice，例如：

```bash
KEYBOARD_VOICE=en-us+f4 cargo run
```

如果选择 `KEYBOARD_TTS_SOURCE=espeak-ng` 但未安装 eSpeak-ng，构建会直接提示错误。默认 Piper 后端在 Piper 和 eSpeak-ng 都缺失时会生成静音占位音频并给出警告。

## Raspberry Pi 构建

```bash
docker buildx build \
  --platform linux/arm64 \
  --target artifact \
  --output type=local,dest=dist .
```

生成的 `dist/keyboard-voice` 可复制到 Pi。Pi 需要运行图形桌面会话，并提供 ALSA 音频设备。

PrtSc、ScrLk、Pause、Caps Lock、Num Lock、Win 等特殊键直接通过 evdev 读取 `/dev/input`，与桌面使用 X11 还是 Wayland 无关，但要求运行用户在 `input` 组中（Raspberry Pi OS 默认用户已在该组；否则执行 `sudo usermod -aG input $USER` 并重新登录）。权限不足时程序照常运行，只是这些键没有响应，启动时会显示提示。

也可以构建运行镜像：

```bash
docker buildx build --platform linux/arm64 --target runtime -t keyboard-voice:arm64 --load .
```

### GitHub Actions 构建并发布

推送以 `v` 开头的 tag 会触发 [release 工作流](.github/workflows/release.yml)：在 GitHub 的 ARM64 runner 上用 `rust:bookworm` 容器原生编译（与 Raspberry Pi OS Bookworm 的 glibc 一致），构建阶段用 Piper 生成语音，并把产物作为 GitHub Release 附件发布。

```bash
git tag v0.1.0
git push origin v0.1.0
```

发布完成后，可以直接在 Pi 上下载（以 v0.1.0 为例）：

```bash
wget https://github.com/caiguanhao/keyboard-voice/releases/download/v0.1.0/keyboard-voice-v0.1.0-linux-aarch64
chmod +x keyboard-voice-v0.1.0-linux-aarch64
./keyboard-voice-v0.1.0-linux-aarch64
```

## 操作

- 按下任意标准键即可显示并朗读；自动重复事件会被忽略。
- 支持 PrtSc、ScrLk、Pause、Caps Lock、Num Lock、Win 键（Linux 上经 evdev 读取）。Fn 键只有在键盘硬件本身会上报扫描码时才有响应。
- 键名上方会显示一个随主题变化的立体键帽图标。
- `Esc` 朗读为 `Escape`；`Insert`、`Delete`、`Page Up`、`Page Down` 显示为 `Ins`、`Del`、`PgUp`、`PgDn`。
- macOS 的 Command 键显示并朗读为 `Command`。
- `Ctrl+Q` 退出程序。
- 右上角按钮切换 Auto、Light、Dark，选择会保存到用户配置目录。
