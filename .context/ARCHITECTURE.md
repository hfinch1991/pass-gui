# Architecture - Pass GUI

## 技术栈
- **Frontend**: Vue 3 + Vite + Pinia
- **Backend**: Rust + Tauri v2
- **Storage**: 基于文件系统的 `pass` 格式（GPG 加密文本）
- **Communication**: Tauri IPC + Native Messaging (STDIO)

## 核心模块设计

### 1. GPG 处理 (src-tauri/src/gpg.rs)
- 使用子进程调用本地 `gpg` 二进制文件。
- **Passphrase Handling**: 通过 `--pinentry-mode loopback` 支持前端输入并缓存密码。
- **macOS Fix**: 强制 `LC_ALL=C` 确保错误信息可被解析。

### 2. 浏览器集成 (Native Messaging)
- **Native Host**: App 可通过 `--native-messaging` 参数进入后台模式。
- **Protocol**: 实现 Browserpass v3 协议，支持 `version`, `list`, `fetch`, `search`, `save` 动作。
- **Wrapper**: 在 macOS 上使用 Shell 脚本中转，解决 Chrome 对 `.app` 路径的权限限制。

### 3. 浏览器扩展 (browser-extension/)
- **UI**: 使用 **Shadow DOM** 技术将插件界面与网页 CSS 彻底隔离。
- **Positioning**: 结合 `visualViewport` 和 `fixed` 定位，支持网页缩放不移位。
- **Features**: 自动识别域名、内联菜单选择、一键采集保存。
