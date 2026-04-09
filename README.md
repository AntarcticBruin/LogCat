# LogCat 🐾

![ScreenShot_2026-04-07_132206_976](https://gitee.com/Duelist/img/raw/master/2026/04/upgit_20260407_1775539676.png)
![image-20260409112653709](https://gitee.com/Duelist/img/raw/master/2026/04/upgit_20260409_1775705214.png)
![ScreenShot_2026-04-07_132432_344](https://gitee.com/Duelist/img/raw/master/2026/04/upgit_20260407_1775539689.png)

**LogCat** 是一款基于 [Tauri 2.0](https://v2.tauri.app/) 和 [Rust](https://www.rust-lang.org/) 构建的高性能、跨平台远程 SSH 日志查看器与文件浏览器。

它旨在解决开发者在查看大型远程服务器日志时的痛点：延迟高、大文件卡顿、以及对现代 SSH 密钥格式支持不佳。

## ✨ 特性

-   **全异步 Rust 后端**：基于 `russh` 纯 Rust SSH 协议栈实现，原生支持 `ed25519` 密钥。
-   **极速日志流**：采用智能 Batching 机制，在高频日志滚动时自动压缩 IPC 通信频率，CPU 占用极低。
-   **智能高亮**：内置高性能正则分段算法（O(1) 内存分配），支持百万级日志行实时高亮。
-   **秒级目录跳转**：引入 SWR (Stale-While-Revalidate) 缓存策略，瞬间切换远程文件夹。
-   **现代化 UI**：使用 Vue 3 + TypeScript 构建，拥有类似 VS Code 的侧边栏布局，支持自动滚动与日志上限管理（默认 5000 行）。
-   **零依赖**：在 Windows 上无需安装 OpenSSH 客户端或任何 C++ 运行库。

## 🚀 快速开始

### 预备工作

确保你的机器上已安装：
- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (建议 v18+)
- [pnpm](https://pnpm.io/) (或 npm / yarn)

### 安装与运行

1.  **克隆仓库**
    ```bash
    git clone https://github.com/AntarcticBruin/LogCat.git
    cd log-cat
    ```

2.  **安装前端依赖**
    ```bash
    pnpm install
    ```

3.  **启动开发模式**
    ```bash
    pnpm tauri dev
    ```

4.  **构建发行版**
    ```bash
    pnpm tauri build
    ```

## 🛠 技术栈

-   **Frontend**: Vue 3, Vite, TypeScript
-   **Backend**: Rust, Tauri 2.0
-   **SSH & SFTP**: [russh](https://github.com/warp-tech/russh), [russh-sftp](https://github.com/warp-tech/russh-sftp)
-   **Runtime**: Tokio (Async Runtime)

## ⚡ 性能优化细节

为了实现极致的顺滑感，本项目在底层做了以下优化：
-   **IPC 批处理**：Rust 后端在读取 `tail` 流时，会缓存 8KB 或等待 30ms 后再向前端 Emit 事件，避免前端被高频渲染请求淹没。
-   **单次扫描高亮**：高亮逻辑摒弃了传统的逐字匹配，改用一次性正则位置排序扫描，极大地降低了长字符串处理时的 GC 压力。
-   **SFTP 路径探测限制**：在加载目录列表时，对未知类型文件的文本探测设置了严格的 RTT 限制，确保在高延迟网络下目录依然能秒开。

## 📄 开源协议

本项目采用 [MIT License](LICENSE) 协议。
