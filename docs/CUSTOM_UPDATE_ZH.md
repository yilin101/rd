# 自编译客户端在线更新

当前实现支持 Windows 和 macOS 客户端通过公开 GitHub Releases 检查、下载并安装自编译版本。Linux、Android 和 iOS 暂不走这条桌面自动安装链路。

## 配置

仓库根目录提供了可复制填写的模板：`custom-config.example.json`。复制为你自己的配置文件后，只需要修改服务器地址、公钥、API 和更新仓库；不要把真实私钥提交到 Git。

在已签名的 `custom.txt` 配置源 JSON 顶层加入：

```json
{
  "app-name": "YourDesk",
  "custom-update-repository": "your-account/your-public-release-repo"
}
```

`custom-update-repository` 只接受 `owner/repository`，不能填写完整 URL。客户端只会访问该仓库的 GitHub Releases，并且只允许从同一个仓库下载更新文件。

注意：上游 `custom.txt` 使用 RustDesk 官方公钥校验签名，普通 JSON 不能直接使用。自建构建系统还需要配置自己的签名公钥和签名工具。

本仓库提供签名工具。首次生成密钥：

```powershell
cargo run --bin sign-custom-client -- generate-key
```

把输出的 `RUSTDESK_CUSTOM_PUBLIC_KEY` 写入 `src/common.rs` 的自定义公钥位置，把 `RUSTDESK_CUSTOM_SECRET_KEY` 只保存到本机环境变量或 CI Secret。然后生成配置：

```powershell
$env:RUSTDESK_CUSTOM_SECRET_KEY = "你的签名私钥或 32 字节种子"
cargo run --bin sign-custom-client -- sign custom-config.json custom.txt
```

不要把签名私钥、`custom-config.json` 或 `custom.txt` 提交到 GitHub。当前客户端仍使用上游固定公钥；在使用自己的密钥前，必须把客户端校验公钥同步改成你生成的公钥。

## GitHub Actions 云端构建

在仓库 **Settings → Secrets and variables → Actions** 新增三个 Repository secrets：

- `CUSTOM_CLIENT_CONFIG_BASE64`：`custom-config.json` 的 Base64 内容。
- `RUSTDESK_CUSTOM_PUBLIC_KEY`：Ed25519 配置签名公钥。
- `RUSTDESK_CUSTOM_SECRET_KEY`：Ed25519 32 字节种子或 64 字节私钥。

配置完后，从 **Actions → Flutter Tag Build → Run workflow** 输入一个新标签（例如 `custom-1.5.0`）即可在 GitHub Windows Runner 编译。安装包会作为该 Release 的附件上传；真实服务器配置和签名私钥不会写入提交记录或构建日志。

## 发布版本

1. 在配置的公开仓库创建 Release，tag 使用可比较的版本号，例如 `1.5.1`。
2. 上传与 tag 版本完全一致的安装包。
3. 确保编译时 `Cargo.toml` 中的客户端版本低于新 Release tag。

Windows Flutter 客户端文件名：

```text
rustdesk-<version>-x86_64.exe
rustdesk-<version>-aarch64.exe
```

具体架构后缀由 `release_arch_suffix()` 决定，应以同一次构建工作流生成的官方命名为准。自定义客户端即使安装来源是 MSI，更新文件仍使用 EXE。

macOS 文件名：

```text
rustdesk-<version>-x86_64.dmg
rustdesk-<version>-aarch64.dmg
```

仓库必须公开。私有 GitHub Release 下载需要认证，而客户端不会也不应内置 GitHub Token。

## 启用检查

保持 `enable-check-update` 开启。若需要后台自动下载安装，还需启用 `allow-auto-update`。未配置 `custom-update-repository` 的品牌客户端会保持上游行为，不检查官方 RustDesk 更新。

## 验收

1. 安装一个较低版本的自编译客户端。
2. 在配置仓库发布较高版本及对应架构安装包。
3. 手动检查更新，确认显示新版本。
4. 确认下载请求只发往配置仓库，安装后版本号已更新。
5. 将 Release 资产临时改名，确认客户端报下载失败且不会执行其他文件。
