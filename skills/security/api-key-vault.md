# APIKey 本地加密存储与解锁流程

## 目标

- APIKey 永不明文落盘
- 查看/保存前必须显式解锁（输入密码）

## 推荐实现（当前项目风格）

- KDF：Argon2（建议 Argon2id；参数可后续固定）
- 对称加密：AES-256-GCM
- nonce：每次加密随机生成 12 bytes
- 存盘格式：`nonce || ciphertext`

## 文件布局（示例）

- `vault/salt.bin`：16 bytes salt
- `vault/vault_check.enc`：加密的校验串，用于验证密码
- `vault/<provider>.enc`：各 provider 的加密 key

## 解锁流程

1. 初始化（首次）：生成 salt -> derive key -> 写入 check
2. 解锁：读取 salt + check -> derive key -> 解密验证
3. 锁定：清除内存中的派生 key

## 安全注意

- 对 provider id 做白名单/slug 化，避免文件名注入。
- 失败信息不要包含敏感内容。
- 如果要更进一步：考虑对内存中的派生 key 做 `zeroize`。
