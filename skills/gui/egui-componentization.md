# egui 子功能组组件化拆分

## 目标

- 将一个大视图拆成独立组件文件（`*Component`），减少冲突并加快编译。
- 保持 UX 不变：左侧导航、中央主视图、右侧信息流。

## 最小步骤

1. 新建组件文件：`crates/vgs/src/components/<name>.rs`
2. 定义：
   - `pub struct <Name>Component { /* ui state */ }`
   - `impl <Name>Component { pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut crate::VgaGuiApp) { ... } }`
3. 在 `crates/vgs/src/components/mod.rs` 导出模块。
4. 在 App struct 中加字段：`<name>_view: <Name>Component`。
5. 在 `match active_view` 中调用该组件渲染。

## UI 布局注意

- 大量按钮/输入行使用 `ui.horizontal_wrapped(...)`，避免在窄窗口下挤压/遮挡右侧信息栏。
- `ScrollArea` 需要显式 `.id_source(...)`，避免重复 ID。

## 借用规则（常见坑）

当调用形如 `self.<component>.ui(ui, self)` 时会触发 Rust 的可变借用冲突。

可用模式：

- 临时 move 出组件：
  - `let mut c = std::mem::take(&mut self.<component>);`
  - `c.ui(ui, self);`
  - `self.<component> = c;`
