# Dorea 开发路线图

## [v0.5.0] - 计划中

### ✨ CLI 美化
- [x] 添加启动 Banner
- [x] 连接状态美化提示
- [x] 命令执行结果美化（颜色、图标）
- [x] JSON 格式化输出
- [x] docs 命令美化
- [x] 提醒用户 docs 可以查文档

### 🚀 功能优化

#### 1. 优化预加载
- [ ] 并发加载数据库，启动速度提升
- [ ] 预加载失败不阻塞启动
- [ ] 添加加载进度日志

#### 2. 嵌入式支持
- [ ] 新增 `embedded` feature
- [ ] 暴露核心 API（DataBaseManager, DataValue）
- [ ] 添加嵌入式使用示例
- [ ] 更新文档

### 🐛 Bug 修复

#### 3. 修复重大 BUG
- [ ] 移除 `panic!`，改用 `Result` 返回错误
- [ ] 修复 WebSocket 连接时的 `unwrap()` panic
- [ ] 密码存储改用哈希（bcrypt/argon2）
- [ ] 修复过期时间判断的类型转换问题

---

## 已完成版本

### [v0.4.0] - 2026-06-08
- Docker 镜像优化，体积减少 95%+
- 新增 Apple Silicon (M1/M2/M3) 支持
- 修复 CI 兼容性问题

### [v0.3.1] - 之前
- 基础 KV 存储功能
- Web Service API
- 命令行工具
