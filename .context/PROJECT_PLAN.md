# Project Plan - Pass GUI

## 项目目标
开发一个基于 Tauri 的 `pass` (Unix standard password manager) 图形化界面客户端，提供跨平台的密码管理体验，并支持浏览器自动填充。

## 关键里程碑
- [x] **阶段 1**: 核心功能实现。列表展示、GPG 解密、Git 同步、条目增删改。
- [x] **阶段 2**: 体验优化。解决 macOS GPG 报错、优化批量导入性能。
- [x] **阶段 3**: 浏览器集成。实现 Native Messaging 协议，开发专用 Chrome 扩展。
- [ ] **阶段 4**: 多平台分发。完善 Windows 11 的构建与自动化安装逻辑。

## 路线图
1. 完善 Windows 下的 GPG 和 Git 环境兼容性测试。
2. 增加多语言支持（i18n）。
3. 支持设置界面管理 GPG 密钥对。
