# ★ Canopus

> *ἀλήθεια — 真理不是被发现的，而是被揭示的。*
> 
> *致虚极，守静笃。吾以观复。— 道德经·十六章*

Canopus 是一个本地优先的个人认知观测系统。它不是聊天助手，不给建议，不做评判。它做一件事：**持续观测你是谁、你如何行动、你的盲点在哪里，并从第三者视角将其反映给你。**

原则不是发明出来的，而是从亲身经历中反复观察和反思后提炼出来的。Canopus 是积累这些原材料并使其可读的基础设施。

---

## 命名

Canopus 是全天第二亮的恒星。数千年来，古代航海者用它在南半球确定自己的位置。

它不告诉你去哪里。**它告诉你你在哪里。**

---

## 核心哲学

- **隐私第一** — 任何情况下，数据不离开设备
- **观测而不干预** — 系统负责反映，不负责处方
- **长期积累** — 单次对话是噪声，数年的数据才是信号
- **诚实优先于舒适** — 第三者视角不能迎合用户
- **完全开源可审计** — 每一行代码对用户可见

---

## 系统架构

```
输入层    日记文字、iPhone 结构化数据
   ↓
处理层    文本标准化、结构化数据转自然语言
   ↓
推理层    Qwen2.5 7B / QwQ 32B via Ollama（本地，Metal GPU 加速）
   ↓
存储层    加密 JSON 画像 + 历史链表
   ↓
访问层    CLI + Web Dashboard（Axum，局域网 / Tailscale）
```

所有计算在用户设备上完成，不向任何外部服务发起网络请求。

---

## 功能

### 数据录入
```bash
canopus journal          # 录入日记（CLI）
canopus task add <title> # 添加任务
canopus task done <id>   # 标记完成
canopus task skip <id>   # 标记跳过（填写原因）
canopus sleep <hours>    # 录入睡眠
canopus screen --today   # 录入屏幕时间
```

### 认知对抗分析
```bash
canopus analyze          # 今日认知对抗分析（默认 Level 2）
canopus analyze --brutal # Level 3 对抗模式
canopus observe          # 提炼今日原始观察
canopus reflect          # 归纳过去 7 天的行为模式
```

### 原则库
```bash
canopus principles list          # 查看所有原则
canopus principles add           # 手动添加候选原则
canopus principles evidence <id> # 添加支撑证据
canopus principles validate <id> # 升级到 validated
canopus principles confirm <id>  # 升级到 confirmed
canopus principles deprecate <id># 标记为已废弃
```

### Web Dashboard
```bash
canopus-dashboard  # 启动本地 Web 界面，自动打开浏览器
```
地址：`http://localhost:7437`

---

## 原则状态机

基于瑞·达利欧《原则》的方法论，每条原则经历完整的生命周期：

```
candidate（候选）
    ↓ 3 条以上独立证据
validated（已验证）
    ↓ 1 次以上实战验证
confirmed（已确认）
    ↓ 不再适用时
deprecated（已废弃）
```

---

## 分析强度

每日分析根据矛盾分（0–1）自动调整输出强度：

| 矛盾分 | 强度 | 风格 |
|--------|------|------|
| < 0.4 | Level 1 | 中性描述 |
| 0.4–0.7 | Level 2 | 直接指出矛盾（默认） |
| ≥ 0.7 | Level 3 | 冷静但锋利，拆穿自我叙述 |

矛盾分由三个指标加权计算：
- 高优先级任务跳过率（40%）
- 屏幕时间生产力占比（35%）
- 睡眠状态修正（25%）

---

## 数据存储

```
~/.canopus/
├── data/
│   ├── journal/        # YYYY-MM-DD.json（日记数组）
│   ├── tasks/          # YYYY-MM-DD.json（任务数组）
│   ├── sleep/          # YYYY-MM-DD.json
│   └── attention/      # YYYY-MM-DD.json（屏幕时间）
├── observations/       # YYYY-MM-DD.json（每日观察）
├── reflections/        # YYYY-MM-DD.json（周反思）
├── principles/         # principle_NNN.json + index.json
└── config.json
```

所有数据为本地 JSON 文件，无数据库，无云同步，可直接读取和编辑。

---

## 技术栈

| 组件 | 技术 | 原因 |
|------|------|------|
| 开发语言 | Rust | 内存安全、性能、可审计性 |
| 模型运行时 | Ollama | 本地模型管理，Metal GPU 加速 |
| 核心模型（开发） | Qwen2.5 7B | 中文理解能力强，7B 适合 M2 16GB |
| 核心模型（生产） | QwQ 32B | 强推理能力，自我反思能力出色 |
| HTTP 服务器 | Axum | 异步 Rust，tokio 生态 |
| CLI 框架 | clap 4 | derive 模式，类型安全 |
| 序列化 | serde + serde_json | Rust 标准 |
| 远程访问 | Tailscale | P2P 加密隧道 |

---

## 硬件要求

| 阶段 | 设备 | 模型 | 推理速度 |
|------|------|------|---------|
| 开发 | MacBook Pro M2 16GB | Qwen2.5 7B | ~15 tok/s |
| 生产 | Mac Mini M2 Pro 32GB | QwQ 32B | ~25 tok/s |

---

## 安装与运行

### 前置条件

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Ollama
brew install ollama

# 设置 Ollama 开机自启
brew services start ollama

# 下载模型
ollama pull qwen2.5:7b
```

### 构建

```bash
git clone https://github.com/yourname/canopus
cd canopus
cargo build --release
```

### 初始化

```bash
# 初始化数据目录
cargo run -- init

# 确认初始化成功
cargo run -- status
```

### 每日使用流程

```bash
# 早上
canopus sleep 7.0 --quality 4
canopus task add "今日最重要的任务" --priority high

# 全天随时
canopus journal --text "今天在想什么..."

# 晚上
canopus screen --today
canopus task done 1
canopus task skip 2 "原因"
canopus analyze

# 启动 Web Dashboard（推荐日常使用）
canopus-dashboard
```

---

## 隐私模型

**保证：**
- 零出站网络请求，不向任何外部服务发送数据
- 不包含任何遥测、分析或使用报告
- 所有数据本地存储
- Ollama 完全运行在 localhost
- 服务器只监听白名单接口

**威胁模型：**

| 威胁 | 缓解措施 |
|------|---------|
| 网络数据泄露 | 设计上零出站连接 |
| 未授权设备访问 | Tailscale 设备认证 + IP 白名单 |
| 物理磁盘访问 | 计划：age 静态加密（路线图第二阶段） |
| 模型向外发送数据 | Ollama 完全运行在 localhost |

---

## 开发路线图

**第一阶段（当前）— 基础**
- [x] CLI 数据录入（journal / tasks / sleep / attention）
- [x] 本地矛盾分计算
- [x] Ollama 推理集成
- [x] `canopus analyze`
- [x] `canopus observe` / `canopus reflect`
- [x] 原则库状态机
- [x] Web Dashboard（日记 + 睡眠）

**第二阶段 — 访问与反思**
- [ ] 用户背景上下文（canopus context）
- [ ] 画像结构与历史链表
- [ ] Tailscale 远程访问
- [ ] 数据加密（age）
- [ ] Dashboard 任务 + 屏幕时间模块

**第三阶段 — 多模态输入**
- [ ] Apple Health XML 导入
- [ ] 语音输入（本地 Whisper）
- [ ] 升级至 QwQ 32B

---

## 哲学注记

> *知人者智，自知者明。— 道德经·三十三章*

你无法不借助镜子看见自己的脸。自我认知中的扭曲之所以不可见，恰恰是因为它们就是你观看世界的镜片。

Canopus 的尝试：不是外部评判，而是你已经存在的真相，以一种你能看见的形式被外化出来。

---

*认识你自己。γνῶθι σεαυτόν.*
