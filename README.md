# Solo

一款保护服务器端口的安全工具

Solo 自动检测当前的公网 IP，通过云服务商的 API 动态调整防火墙规则，并将指定端口（例如 SSH/22）的来源 IP 锁定到当前 IP。

经过两年的开发，Solo 的前身 [qcip](https://github.com/cnlancehu/qcip) 项目已趋于完善。

然而，考虑到 Golang 在性能优化方面的局限性，以及各云服务提供商 SDK 库过于庞大的问题，我们决定用 Rust 重写整个项目，并将其命名为 **Solo**。

## 注意
**Solo 仍处于早期开发阶段。**

可能存在 bug 或功能不完善，请谨慎使用。

## 支持的云服务商
| 服务商 |         云产品          |
| :----: | :---------------------: |
| 腾讯云 | 云服务器 轻量应用服务器 |
| 阿里云 | 云服务器 轻量应用服务器 |

我们会尽力适配其他服务商。

如果你有其他云服务商的服务器，并希望帮助我们适配，请通过 issue 与我们联系。

## 指南

### 下载
请前往 [下载页面](https://solo.lance.fun/zh/download/) 下载

### 文档
见 [Solo Doc](https://solo.lance.fun/)

> [!IMPORTANT]
> 文档尚未完善，你可在 [文档仓库](https://github.com/cnlancehu/solo-doc) 提交代码参与贡献。