# htbox init 命令设计

## 背景

htbox 目前在首次使用时自动创建配置目录和默认文件，但缺少交互式初始化流程。用户无法自定义配置，且存在环境变量加载等问题。

## 目标

1. 提供交互式初始化向导，让用户首次使用时配置基本选项
2. 明确初始化状态，避免自动创建带来的不确定性
3. 支持后续重置配置

## 设计

### 命令结构

```bash
htbox init          # 交互式初始化
htbox init --reset  # 重置配置（会提示确认）
htbox init --force  # 使用默认配置强制初始化
```

### 初始化流程

```
$ htbox init

欢迎使用 htbox！首次运行将进行初始化配置。

htbox 根目录: /root/.htbox

? 时区 (default: Asia/Shanghai): 
? 默认后端 (auto/systemd/cron, default: auto): 
? 使用用户级 systemd (yes/no, default: no): 
? 日志级别 (debug/info/warn/error, default: info): 
? 是否添加全局环境变量 (yes/no, default: no): 
  ? KEY: DATABASE_URL
  ? Value: postgres://localhost:5432/mydb
  ? 继续添加 (yes/no): no

初始化完成！
```

### 配置文件 (config.toml)

新增 `version` 字段标识初始化版本：

```toml
version = "1.0.0"

[general]
  work_dir = "~/.htbox"

[backend]
  force = "auto"
  user_level = false

[logging]
  level = "info"

[env]
  global_file = "~/.htbox/envs/global.env"
```

### 对现有命令的影响

| 命令 | 变更 |
|------|------|
| `htbox service add` | 检查初始化状态，未初始化提示运行 init |
| `htbox service list` | 同上 |
| `htbox cmd *` | 同上 |
| 其他命令 | 同上 |

### 兼容性

1. 已有 config.toml 的用户 → 跳过初始化，直接使用
2. 无 config 但有 services 目录 → 提示初始化
3. `--force` 参数 → 强制使用默认配置（自动化场景）

## 实现计划

1. 修改 config.toml 添加 version 字段
2. 新增 `htbox init` 命令
3. 修改 `Config::load()` 添加初始化状态检查
4. 为现有命令添加 `--force` 参数
5. 更新文档
