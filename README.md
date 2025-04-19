# Solo
**English** | [简体中文](README-zh.md)

Solo is a lightweight security tool that focuses on protecting the core ports of the server. It can automatically detect your current public IP, dynamically adjust firewall rules through the cloud platform API, and lock the access rights of the specified port (such as SSH/22) to the administrator's real-time IP.

After two years of development, Solo's predecessor [qcip](https://github.com/cnlancehu/qcip) project has become more complete.

However, considering the limitations of Golang in performance optimization and the problem that the SDKs of various cloud service providers are too large, we decided to rewrite the entire project in Rust and named it Solo.


## Warning
**Solo is still in the early stages of development.**

There might be bugs or incomplete features. Please use it with caution.

## Supported Cloud Service Providers
| Service Provider |     Product     |
| :--------------: | :-------------: |
|  Tencent Cloud   | CVM, Lighthouse |
|  Alibaba Cloud   |    ECS, Swas    |

We are currently adapting to other cloud service providers, if you have a server of non-supported service provider and want to help us adapt it, please contact us.

## Guide
