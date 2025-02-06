# Solo
Solo 是一款轻量级安全工具，专注守护服务器核心端口。它能自动检测您当前的公网IP，并通过云平台API动态调整防火墙规则，将指定端口（如SSH/22）的访问权限锁定为管理员实时IP。

经过两年的开发历程，Solo 的前身 [qcip](https://github.com/cnlancehu/qcip) 项目已经趋于完善。

然而，考虑到 Golang 在性能优化方面的局限性，以及各云服务厂商 SDK 过于庞大的问题，

我们决定使用 Rust 重写整个项目，并将其命名为 Solo。

## 开发进展
已完成
- 基本的服务商 SDK 实现
- 包含 阿里云云服务器、轻量，腾讯云云服务器、轻量

正在开发
- 命令行应用
- 网页 UI

大概还需要一段时间，请耐心等待

## 参与讨论
如果您有任何想法或建议，在 [Discussion](https://github.com/cnlancehu/solo/discussions/new?category=ideas) 区域与我们交流。
