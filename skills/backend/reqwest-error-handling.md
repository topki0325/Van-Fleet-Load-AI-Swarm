# reqwest 错误处理与超时策略

## 基本建议

- 为连接与请求设置超时：`connect_timeout` + `timeout`
- 非 2xx：尽量读取 body 作为错误信息（先缓存 status，避免 move 问题）
- 对 JSON 解析错误单独报错，避免把所有错误混在一起

## Rust 模式（避免 move/borrow）

```rust
let status = response.status();
if !status.is_success() {
    let body = response.text().await.unwrap_or_else(|_| "(no body)".to_string());
    return Err(format!("HTTP {}: {}", status, body));
}
```

## UI 线程注意

- GUI 场景不要在渲染帧里 `block_on` 长请求；用后台任务 + channel 回写。
