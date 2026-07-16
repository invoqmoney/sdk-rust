# invoq Rust SDK

[English](../README.md) · [Bahasa Indonesia](./README.id.md) · [Español](./README.es-419.md) · [Français](./README.fr.md) · [Português](./README.pt-BR.md) · [Tiếng Việt](./README.vi.md) · [Türkçe](./README.tr.md) · [ไทย](./README.th.md) · **简体中文** · [繁體中文](./README.zh-Hant.md)

> 本文是英文版 README 的简体中文翻译；若表述有出入，以[英文版](../README.md)为准。

面向 invoq 服务端 API 和 webhook 验签的 Rust SDK。

本 crate 只能在你的服务端使用。它会接收密钥，绝不能被编译进客户端应用。

## 服务端 SDK

用下面任意一种语言，都能从你的后端创建账单、验证 webhook——REST API 和 webhook 签名完全一致。本仓库是 Rust SDK。

| 语言 | 仓库 |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)（`@invoq/server`） |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **本仓库** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

无论后端选哪种语言，浏览器这一侧都一样：**`@invoq/checkout`**（JavaScript，位于 [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)）为任意前端打开嵌在页面里的收银台弹窗。

## 安装

```toml
[dependencies]
invoq = "0.1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

本 SDK 基于 `reqwest` 和异步 Rust。

需要 Rust 1.86 或更高版本。

## 获取密钥

1. 登录 [invoq 商户后台](https://app.invoq.money)，创建一个项目。
2. 在 **API keys** 页面创建一把密钥（secret key）。测试密钥以 `sk_test_` 开头，正式密钥以 `sk_live_` 开头；密钥的模式决定开出的账单是测试单还是正式单。
3. 在项目的 **webhooks** 设置里保存你的 webhook URL。对应模式的 webhook 签名密钥（`whsec_...`）只在首次启用 webhook 时展示一次，记得马上存好。webhook URL 必须是公网可访问的 HTTPS 地址。

把密钥加进服务端环境变量：

```sh
INVOQ_SECRET_KEY=sk_test_...
```

如果你要处理 webhook，也请存好 webhook 签名密钥：

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

先用测试密钥跑通，上线时再换成正式密钥和配套的正式 webhook 签名密钥。

## 创建客户端

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

生产环境的 API origin 默认值：

```text
https://api.invoq.money
```

开发时可以覆盖 API origin：

```rust,no_run
use invoq::{Invoq, InvoqOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::with_options(
        std::env::var("INVOQ_SECRET_KEY")?,
        InvoqOptions::default().api_origin("http://localhost:8787"),
    )?;
    Ok(())
}
```

`api_origin` 必须是完整的 `http` 或 `https` origin，不能带路径、查询串、hash、用户名或密码。SDK 会在其后拼接 `/v1/...` 资源路径。

默认的 HTTP 客户端使用 10 秒的请求超时。可以用 `InvoqOptions::timeout_ms(...)` 修改：

```rust,no_run
use invoq::{Invoq, InvoqOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::with_options(
        std::env::var("INVOQ_SECRET_KEY")?,
        InvoqOptions::default().timeout_ms(20_000),
    )?;
    Ok(())
}
```

如果需要不同的传输层设置，可以用 `InvoqOptions::http_client(...)` 传入自定义的 `reqwest::Client`。`timeout_ms` 仍然控制 SDK 每次请求的超时。

## 账单

创建账单：

```rust,no_run
use invoq::{CreateInvoiceInput, Invoq, InvoiceCurrency};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;

    let invoice = invoq
        .invoices
        .create(
            CreateInvoiceInput::new("149")
                .currency(InvoiceCurrency::Usd)
                .description("SaaS boilerplate")
                .reference_id("order_1234")
                .return_url("https://example.com/orders/order_1234"),
        )
        .await?;

    println!("{invoice:?}");
    Ok(())
}
```

不需要时，可以省略 `.description(...)`、`.reference_id(...)` 或 `.return_url(...)`。未设置的可选请求字段不会出现在 JSON 里。用 `.without_return_url()` 可以发送 `return_url: null`，主动放弃项目默认的 return URL。

金额要由服务端决定，不要相信客户端传来的金额。`amount` 是 `"0.01"` 到 `"999.99"` 之间的十进制美元字符串，最多两位小数，比如 `"129"` 或 `"129.99"`。

用 `reference_id` 把 `invoice.paid` webhook 对应回你的订单。它还让创建操作可以放心重试：用相同的 `reference_id` 和相同的账单条款再次创建，返回的是已有账单而不是重复开单；条款不同则会报 `409 reference_id_conflict` API 错误。

查询账单：

```rust,no_run
use invoq::Invoq;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    let invoice = invoq.invoices.get("inv_123").await?;

    println!("{invoice:?}");
    Ok(())
}
```

`invoices.get()` 返回收银台使用的公开账单结构。它包含面向收银台的字段，例如 `amount_paid`、`amount_due`、`amount_overpaid`、`payment_status`、`project`、`deposit_address`、`monitoring_ends_at`、`monitoring_status`、`transfers` 和 `direct_onchain_rails`，但不包含 `reference_id`。如果需要商户侧的参考号，请使用创建账单的响应或 `invoice.paid` webhook。

创建模拟付款：

```rust,no_run
use invoq::{CreateTestPaymentInput, Invoq};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    let paid_invoice = invoq
        .invoices
        .create_test_payment(
            "inv_123",
            CreateTestPaymentInput::new("149").reference_id("test_payment_001"),
        )
        .await?;

    println!("{paid_invoice:?}");
    Ok(())
}
```

`create_test_payment()` 只对用 `sk_test_` 密钥创建的账单有效。累计付款达到账单金额时，账单变为 `paid`，invoq 会向你的测试 webhook URL 发送一条真实签名的 `invoice.paid` webhook。也可以只付部分金额，账单会变成 `partially_paid`。

不需要时可以省略 `.reference_id(...)`；不要给可选的请求字符串传 `null`。未设置的可选字符串不会出现在请求 JSON 里。

SDK 直接返回响应里的 `data` 对象。

响应里的金额都经过规范化：用 `"129"` 创建，账单返回 `amount: "129.0000"`。比较金额请按数值比，不要按字符串比。`amount_due` 按 `max(amount - amount_paid, 0)` 派生，使用和 `amount_paid` 相同的 18 位小数 scale；`amount_overpaid` 与它互为镜像，即 `max(amount_paid - amount, 0)`，所以你不必自己做减法。`monitoring_status` 取值 `active` 或 `ended`——一旦变为 `ended`，收款地址就不再被监控——而 `transfers` 是已确认的链上收款记录（每一项都含 `tx_hash`、`amount` 和 `explorer_tx_url`）。测试账单里两者分别为 `null` / `[]`。

## 托管收银页

每张账单还自带一个托管收银页：

```text
https://pay.invoq.money/<invoice id>
```

页内收银台弹窗不合适的场景，把链接发出去或直接跳转过去就行。

## Webhooks

把原始请求体传给 `verify_webhook`。验签之前不要先解析 JSON 再重新序列化。

```rust,no_run
use invoq::{invoice_paid_event, verify_webhook};

fn handle_webhook(
    raw_body: &[u8],
    signature_header: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let event = verify_webhook(
        raw_body,
        signature_header,
        std::env::var("INVOQ_WEBHOOK_SECRET")?.as_str(),
    )?;

    if let Some(invoice_paid) = invoice_paid_event(&event) {
        let Some(order_id) = invoice_paid.data.invoice.reference_id.as_deref() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing invoice reference_id for fulfillment.",
            )
            .into());
        };

        println!("{order_id}");
    }

    Ok(())
}
```

`verify_webhook` 接受 `http::HeaderMap`、一个包含 `invoq-signature` 头部值的字符串，或者 header 键不区分大小写的受支持 map 类型。

webhook 验签失败会返回 `InvoqSignatureVerificationError`。

用 `invoice.paid` webhook 在你的服务端履约订单。`invoice_paid_event(&event)` 返回可履约的已付款事件的类型化数据；如果你只需要一个布尔值，`is_invoice_paid(&event)` 对同样的事件返回 true。用账单的 `reference_id` 找到并履约对应的订单。这两个辅助函数接受可视为已付款的账单状态（`paid`、`settling` 或 `settled`），并拒绝 `review_required`。`review_required` 账单暂时还不会发出 `invoice.paid` webhook。

投递失败会重试，所以请按 `reference_id` 或账单 `id` 幂等履约，让重复投递变成空操作。尽快返回 2xx；其他任何状态码都算投递失败。

SDK 允许 5 分钟的时间戳容差。投递失败后每次重试都会重新签名，所以正常的重试投递仍然能在这个时间窗内验签通过。签名头格式是：

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## 错误处理

```rust,no_run
use invoq::{CreateInvoiceInput, Invoq, InvoqError};

async fn handle_error(invoq: Invoq) -> Result<(), Box<dyn std::error::Error>> {
    match invoq.invoices.create(CreateInvoiceInput::new("1000")).await {
        Ok(invoice) => println!("{invoice:?}"),
        Err(InvoqError::Api(error)) => {
            eprintln!("status: {}", error.status);
            eprintln!("code: {:?}", error.code);
            eprintln!("fields: {:?}", error.fields);
            eprintln!("meta: {:?}", error.meta);
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}
```

API 入参校验、连接失败、超时以及响应解析失败都会返回 `InvoqError`。

## 许可证

基于 MIT 许可证授权。
