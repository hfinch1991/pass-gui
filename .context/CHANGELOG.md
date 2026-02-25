# Changelog - Pass GUI

## [2026-02-24] - 浏览器集成与性能修复

### Added
- **Browser Extension**: 开发了专用的 Chrome 扩展，支持 Shadow DOM 隔离和自动缩放定位。
- **Auto-Save**: 支持从浏览器直接采集并保存当前页面的账号密码。
- **Passphrase Cache**: 前端支持 GPG 密码输入对话框及内存级缓存。
- **Windows Support**: 增加了 Windows 注册表 Manifest 安装逻辑的准备代码。

### Fixed
- **Startup Crash**: 修复了 `tauri.conf.json` 中 `dialog` 插件配置错误导致的启动 panic。
- **Silent Mode**: 彻底修复了浏览器调用 App 时意外弹出 GUI 前端窗口的问题。
- **Search Filtering**: 修复了 GUI 搜索框无法通过路径匹配条目的问题。
- **Import Performance**: 优化了 CSV 导入逻辑，采用批量提交 Git 和单一 ID 读取，解决界面无响应问题。
- **GPG Detection**: 优化了 macOS 路径下的 GPG 二进制文件探测。

### Changed
- **UI**: 将插件菜单宽度增加至 300px，提升可读性。
- **Security**: 移除了应用包的 macOS quarantine 隔离属性。
