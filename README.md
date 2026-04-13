# htbox

Linux CLI tool for service and command management.

## 功能特性

### 服务管理
- **常驻服务 (daemon)** - 持续运行的后台服务
- **一次性服务 (onetime)** - 每次系统启动时执行一次
- 支持 systemd (优先) 和 cron 后端自动切换
- 服务状态监控、日志查看、环境变量管理

### 快捷命令
- 参数化命令模板 (Tera 模板引擎)
- 命令执行日志记录
- 参数校验和默认值支持

## 安装

```bash
cargo install htbox
```

或从源码编译:

```bash
git clone https://github.com/haitaocq/htbox.git
cd htbox
cargo build --release
```

## 使用

### 服务管理

```bash
# 列出所有服务
htbox service list

# 添加服务
htbox service add

# 启动/停止/重启
htbox service start <name>
htbox service stop <name>
htbox service restart <name>

# 开机自启
htbox service enable <name>
htbox service disable <name>

# 查看状态
htbox service status <name>

# 查看日志
htbox service logs <name> -n 50
htbox service logs <name> -f

# 删除服务
htbox service remove <name> --force
```

### 快捷命令

```bash
# 列出命令
htbox cmd list

# 添加命令
htbox cmd add <name> --command "echo {{message}}"

# 执行命令
htbox cmd run <name> --message "Hello"

# 删除命令
htbox cmd remove <name> --force
```

## 配置

配置文件位于 `~/.htbox/config.toml`:

```toml
[backend]
force = "auto"        # auto | systemd | cron
user_level = false    # 使用用户级 systemd

[logging]
level = "info"
```

## 项目结构

```
htbox/
├── src/
│   ├── cli.rs          # CLI 定义
│   ├── commands/       # 命令实现
│   ├── config/        # 配置管理
│   ├── backend/       # 后端实现 (systemd/cron)
│   ├── runtime/        # 运行时
│   └── state/          # 状态管理
└── tests/              # 测试
```

## 测试

```bash
cargo test
```

## License

MIT