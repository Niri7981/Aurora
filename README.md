# AuroraPulse

# Aurora

一个围绕你本地 `Gemma4` 模型搭建的小型 agent 助理项目骨架。

当前定位不是“完整 Siri 成品”，而是一个清晰、可继续扩展的本地 agent 工程：

- `Gemma4` 作为本地大脑，负责理解自然语言和产出结构化意图
- `Spotify` 作为第一批工具能力，负责搜索与播放控制
- `CLI` 作为第一版交互入口
- 后续可继续接 `Whisper`、`TTS`、唤醒词和后台常驻

## 当前文件结构

```text
AuroraPulse/
├── agent.py
├── README.md
├── .env.example
└── aurorapulse/
    ├── core/
    │   ├── agent.py
    │   └── settings.py
    ├── integrations/
    │   ├── llm/
    │   │   └── gemma4_ollama.py
    │   └── music/
    │       └── spotify.py
    └── interfaces/
        └── cli.py
```

## 每层职责

- `aurorapulse/core`
  - 放 agent 编排逻辑
  - 负责把模型决策和工具执行串起来

- `aurorapulse/integrations/llm`
  - 放本地模型接入
  - 当前默认接 `Ollama + gemma4:e4b`
  - 以后你也可以替换成你自己的 `my-gemma4-e4b:latest`

- `aurorapulse/integrations/music`
  - 放 Spotify 工具适配层
  - 以后也可以再加 Apple Music、YouTube Music

- `aurorapulse/interfaces`
  - 放用户交互入口
  - 当前先用 CLI
  - 后续可以再加 `voice.py`、`daemon.py`

## 本地 Gemma4 怎么放进这个项目

你现在本机已经有这些模型：

- `gemma4:e4b`
- `my-gemma4-e4b:latest`

项目默认通过环境变量选择模型：

```env
OLLAMA_MODEL=gemma4:e4b
OLLAMA_URL=http://127.0.0.1:11434
```

如果你想直接用你自己的版本，只需要改成：

```env
OLLAMA_MODEL=my-gemma4-e4b:latest
```

也就是说，这个项目不是“把 Gemma4 拷进来”，而是把它作为本地推理服务接入。

## 你后面推荐的开发顺序

1. 先把 CLI + Gemma4 + Spotify 跑通
2. 再加 `voice` 接口层
3. 给 `voice` 接本地 STT
4. 给输出接 TTS
5. 最后再做唤醒词和常驻后台

## 运行思路

当前项目正在迁移到 Rust CLI，推荐启动方式是：

```bash
cargo run -- .
```

如果后面安装为本地命令，则会是：

```bash
aurora .
```

这里的 `.` 表示“以当前目录作为工作区启动”。

它会走这条链路：

1. 用户输入一句话
2. `Gemma4Planner` 解析成结构化动作
3. `AuroraAgent` 决定调用什么工具
4. `SpotifyController` 执行 API 调用
5. CLI 返回结果

## 环境变量

先复制：

```bash
cp .env.example .env
```

然后填：

- `SPOTIFY_CLIENT_ID`
- 可选修改 `OLLAMA_MODEL`

Redirect URI 保持：

- `http://127.0.0.1:8888/callback`

## 下一步最值得补的文件

如果你继续写这个项目，下一批最自然的新增文件会是：

- `aurorapulse/interfaces/voice.py`
- `aurorapulse/integrations/stt/whisper.py`
- `aurorapulse/integrations/tts/macos_say.py`
- `aurorapulse/core/session.py`
- `aurorapulse/core/tools.py`
