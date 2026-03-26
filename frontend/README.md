# Canopus — Next.js Dashboard

Personal dashboard for tracking thoughts, sleep, and tasks. Next.js frontend proxies all API calls to a Rust backend.

## 启动方式

### 1. 启动 Rust 后端

在项目根目录（`/path/to/Canopus`）运行：

```bash
cargo run --release --bin canopus-dashboard
```

Rust 后端默认监听 `http://localhost:7437`。

### 2. 启动 Next.js 前端

在 `frontend/` 目录下运行：

```bash
pnpm dev
```

打开浏览器访问 `http://localhost:3000`。

## 技术栈

- **框架**: Next.js 15 (App Router)
- **样式**: Tailwind CSS v4 + shadcn/ui
- **数据请求**: SWR
- **后端**: Rust（canopus-dashboard binary）

## API 路由

Next.js API 路由作为代理，将所有请求转发到 Rust 后端：

| 路径 | 转发到 |
|------|--------|
| `/api/journal` | `http://localhost:7437/api/journal` |
| `/api/journal/[id]` | `http://localhost:7437/api/journal/:id` |
| `/api/sleep` | `http://localhost:7437/api/sleep` |
| `/api/tasks` | `http://localhost:7437/api/tasks` |
| `/api/tasks/[id]` | `http://localhost:7437/api/tasks/:id` |

## 功能

- **日记**: 写日记、情绪标记、标签、按日期分组显示、删除
- **睡眠**: 记录入睡/起床时间和质量评分、7天睡眠柱状图（含 Y 轴时间刻度、点击 popover 显示详情）
- **任务**: 四象限矩阵（紧急/重要）、完成/跳过/删除、Execution Index 仪表盘
- **概览**: 汇总今日记录数、平均睡眠、任务完成率、执行指数
