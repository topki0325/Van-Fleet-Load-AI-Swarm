# 组件规则（GUI 子功能组组件化）

目标：

- 子功能组作为“组件”加载到主 GUI 中
- 修改 GUI 时尽量只重编译 GUI crate（`vgs`），减少对 Tauri/其它部分的编译影响

本项目当前采用 workspace 拆分：

- `crates/vas-core`：后端 + 数据模型（可被 Tauri 与 GUI 复用）
- `crates/vgs`：原生 GUI（egui/eframe）

这样你在开发 GUI 时使用 `cargo build -p vgs`/`cargo run -p vgs`，只会编译 `vgs + vas-core`。

## 1) 组件定义

每个子功能组（例如 Providers/Resources/API 管理）都应当是一个独立的 `struct`，并拥有自己的 UI 状态。

组件结构建议：

```rust
pub struct ProvidersComponent {
    // UI state (inputs, toggles, last json, status, etc.)
}

impl ProvidersComponent {
    pub fn new() -> Self { Self { /* ... */ } }

    pub fn ui(&mut self, ui: &mut egui::Ui, core: &mut CoreFacade) {
        // draw controls, call core services, update self state
    }
}
```

规则：

- **组件只维护自身状态**：输入框内容、展开/收起、最近一次响应 JSON、错误提示等
- **组件不直接持有 tokio runtime**：统一由 App/Facade 调用（避免多 runtime 与 borrow 问题）
- **组件不跨组件读写字段**：跨组件通信通过 App 层的“共享只读信息”或显式事件/回调

## 2) CoreFacade（组件访问后端的统一入口）

为了避免组件到处拿 `Arc<BackendServices>`/runtime，建议在 GUI crate 定义一个 facade：

```rust
pub struct CoreFacade {
    pub services: std::sync::Arc<vas_core::backend::BackendServices>,
    pub runtime: tokio::runtime::Runtime,
}
```

组件通过 `core.runtime.block_on(async { ... })` 去调用 `services.*`。

规则：

- 后端调用要集中在组件内的按钮点击处（不要每帧都 block_on）
- 返回内容尽量序列化为 JSON 文本再展示（避免复杂表格先带来大量 UI 代码）

## 3) 组件注册与加载

主 App 只负责：

- 顶部菜单（VS Code 风格）
- 左侧导航（功能组列表）
- 中间区域：根据当前 ActiveView 渲染对应组件
- 右侧信息瀑布流：渲染全局只读信息（swarm/agents/projects/leases/tasks）

建议的加载方式：

```rust
pub struct App {
    pub active_view: ActiveView,
    pub providers: ProvidersComponent,
    pub resources: ResourcesComponent,
    pub api_manager: ApiManagerComponent,
}

match self.active_view {
    ActiveView::Providers => self.providers.ui(ui, &mut self.core),
    ActiveView::Resources => self.resources.ui(ui, &mut self.core),
    // ...
}
```

## 4) 编译/依赖规则（节约时间）

为了让 GUI 改动不触发 Tauri 重编译：

- GUI 相关依赖（`eframe`/`egui-chinese-font`）只放在 `crates/vgs`
- 后端与模型只放在 `crates/vas-core`
- Tauri 仅留在根包（`vangriten-ai-swarm`）

常用命令：

- 只编译 GUI：`cargo build -p vgs`
- 运行 GUI：`cargo run -p vgs`
- 编译 Tauri（根包）：`cargo build -p vangriten-ai-swarm`

## 5) 命名规范

- `*Component`：组件 struct（一个子功能组）
- `ActiveView`：左侧导航枚举
- `*_json`：存放展示用的 JSON 字符串
- `*_status`：短状态文本（成功/失败）
