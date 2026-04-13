# htbox CLI 功能点列表

> 日期: 2026-04-13
> 状态: 待实现

## 功能点汇总

| 序号 | 模块 | 功能点 | 说明 |
|------|------|--------|------|
| F1.1 | 项目基础设施 | 创建 Rust 项目脚手架 | Cargo.toml, 目录结构 |
| F1.2 | 项目基础设施 | 配置核心依赖 | clap, tokio, toml, tracing, thiserror, anyhow, tera |
| F1.3 | 项目基础设施 | 项目目录结构搭建 | 按设计文档创建 src/ 模块 |
| F2.1 | 错误处理模块 | 定义 Error 枚举类型 | ServiceNotFound, ScriptNotFound, PermissionDenied 等 |
| F2.2 | 错误处理模块 | 实现错误链 | thiserror + anyhow |
| F3.1 | 配置管理 | 全局配置解析 | config.toml 加载 |
| F3.2 | 配置管理 | 服务配置解析 | service.toml 加载 |
| F3.3 | 配置管理 | 全局环境变量加载 | global.env 解析 |
| F3.4 | 配置管理 | 服务环境变量加载 | .env 解析 |
| F4.1 | 后端实现 | 后端自动检测 | systemd vs cron 自动选择 |
| F4.2 | 后端实现 | 容器环境检测 | /proc/1/cgroup, /.dockerenv |
| F4.3 | 后端实现 | Systemd unit 文件生成 | 生成 .service 文件 |
| F4.4 | 后端实现 | Systemd 基础操作 | start/stop/restart/enable/disable |
| F4.5 | 后端实现 | Cron daemon 管理 | PID 文件 + 后台运行 |
| F4.6 | 后端实现 | Cron onetime 管理 | @reboot 任务 |
| F4.7 | 后端实现 | Cron crontab 管理 | 备份/恢复 |
| F5.1 | CLI 命令结构 | CLI 入口定义 | clap derive |
| F5.2 | CLI 命令结构 | help/version 支持 | --help, --version |
| F6.1 | 服务管理命令 | start | 立即启动服务进程 |
| F6.2 | 服务管理命令 | stop | 停止服务进程 |
| F6.3 | 服务管理命令 | restart | 重启服务 |
| F6.4 | 服务管理命令 | enable | 设置开机自启 |
| F6.5 | 服务管理命令 | disable | 取消开机自启 |
| F6.6 | 服务管理命令 | status | 查看状态+日志+资源 |
| F6.7 | 服务管理命令 | list | 列出所有服务 |
| F6.8 | 服务管理命令 | add | 添加服务 (交互式/参数) |
| F6.9 | 服务管理命令 | edit | 编辑服务配置 |
| F6.10 | 服务管理命令 | remove | 删除服务 (含确认) |
| F6.11 | 服务管理命令 | logs | 查看日志 (-n/-f/--errors) |
| F6.12 | 服务管理命令 | env list | 列出服务环境变量 |
| F6.13 | 服务管理命令 | env add | 添加服务环境变量 |
| F6.14 | 服务管理命令 | env remove | 删除服务环境变量 |
| F7.1 | 快捷命令管理 | list | 列出所有快捷命令 |
| F7.2 | 快捷命令管理 | add | 添加快捷命令 (交互式) |
| F7.3 | 快捷命令管理 | run | 执行快捷命令 |
| F7.4 | 快捷命令管理 | edit | 编辑快捷命令 |
| F7.5 | 快捷命令管理 | remove | 删除快捷命令 |
| F7.6 | 快捷命令管理 | 参数解析 | --key=value, --key value |
| F7.7 | 快捷命令管理 | 参数校验 | required, default |
| F7.8 | 快捷命令管理 | Tera 模板渲染 | 变量替换/条件/循环 |
| F7.9 | 快捷命令管理 | 执行日志 | 记录 START/output/END |
| F8.1 | 运行时管理 | 状态管理 | state.json 操作 |
| F8.2 | 运行时管理 | PID 文件管理 | 读写 run/pid |
| F8.3 | 运行时管理 | 资源查询 | CPU, Memory, Threads |
| F8.4 | 运行时管理 | 运行时间计算 | uptime 计算 |
| F9.1 | 辅助功能 | 脚本模板生成 | 创建 service.sh 模板 |
| F9.2 | 辅助功能 | 交互式创建 | 提示向导 |
| F9.3 | 辅助功能 | 日志实时跟踪 | -f 参数 |
| F9.4 | 辅助功能 | 错误日志过滤 | --errors 参数 |
| F9.5 | 辅助功能 | user-level systemd | 配置支持 |

---

## 按模块统计

| 模块 | 功能点数 |
|------|----------|
| 项目基础设施 | 3 |
| 错误处理模块 | 2 |
| 配置管理 | 4 |
| 后端实现 | 7 |
| CLI 命令结构 | 2 |
| 服务管理命令 | 14 |
| 快捷命令管理 | 9 |
| 运行时管理 | 4 |
| 辅助功能 | 5 |
| **总计** | **50** |

---

## 实现优先级

### P0 - 核心功能 (必须实现)
1. 项目脚手架 + 依赖配置
2. 错误处理模块
3. 配置管理 (全局 + 服务)
4. 后端检测 + systemd 实现
5. CLI 入口 + help/version
6. 服务管理命令 (start/stop/restart/status/list)
7. 快捷命令 (list/run)

### P1 - 重要功能
1. Cron 后端实现
2. enable/disable 功能
3. 服务 add/edit/remove
4. 快捷命令 add/edit/remove
5. 状态管理

### P2 - 辅助功能
1. 交互式创建
2. logs -f 实时跟踪
3. 资源监控
4. env 管理
5. 参数校验

---

## 关联设计文档

- 文档: `docs/superpowers/specs/2026-04-13-htbox-cli-rust-design.md`
- 版本: Rust 版

## 变更记录

| 日期 | 变更 |
|------|------|
| 2026-04-13 | 初始版本，基于设计文档拆分功能点 |