# invoq Rust SDK

[English](../README.md) · [Bahasa Indonesia](./README.id.md) · [Español](./README.es-419.md) · [Français](./README.fr.md) · [Português](./README.pt-BR.md) · [Tiếng Việt](./README.vi.md) · [Türkçe](./README.tr.md) · [ไทย](./README.th.md) · [简体中文](./README.zh-Hans.md) · **繁體中文**

> 本文是英文版 README 的繁體中文翻譯；若表述有出入，以[英文版](../README.md)為準。

適用於 invoq 伺服器 API 與 webhook 驗證的 Rust SDK。

這個 crate 只能在你的伺服器上使用。它會接受私密金鑰，絕不能編譯進用戶端應用程式。

## 伺服器端 SDK

用下面任一種語言，都能從你的後端建立帳單、驗證 webhook——REST API 和 webhook 簽章完全一致。本倉庫是 Rust SDK。

| 語言 | 倉庫 |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)（`@invoq/server`） |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **本倉庫** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

無論後端選哪種語言，瀏覽器這一側都一樣：**`@invoq/checkout`**（JavaScript，在 [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)）為任意前端打開嵌在頁面裡的結帳彈窗。

## 安裝

```toml
[dependencies]
invoq = "0.1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

本 SDK 使用 `reqwest` 與非同步 Rust。

需要 Rust 1.86 或更新版本。

## 取得金鑰

1. 登入 [invoq 商家後台](https://app.invoq.money)，建立一個專案。
2. 在 **API keys** 頁面建立一組私密金鑰（secret key）。測試金鑰以 `sk_test_` 開頭，正式金鑰以 `sk_live_` 開頭；用哪種金鑰，決定開出的帳單是測試單還是正式單。
3. 在專案的 **webhooks** 設定裡儲存你的 webhook URL。對應模式的 webhook 簽章金鑰（`whsec_...`）只在首次啟用 webhook 時顯示一次——記得馬上存好。webhook URL 必須是可公開存取的 HTTPS 網址。

把私密金鑰加進伺服器的環境變數：

```sh
INVOQ_SECRET_KEY=sk_test_...
```

如果你要處理 webhook，也把 webhook 簽章金鑰存起來：

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

先用測試金鑰開始，上線時再換成正式金鑰和對應的正式 webhook 簽章金鑰。

## 建立用戶端

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

正式環境的 API origin 預設值：

```text
https://api.invoq.money
```

開發時可以覆寫 API origin：

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

`api_origin` 必須是完整的 `http` 或 `https` origin，不能帶路徑、查詢字串、hash、使用者名稱或密碼。SDK 會在其後接上 `/v1/...` 資源路徑。

預設的 HTTP 用戶端使用 10 秒的請求逾時。要更改的話，請用 `InvoqOptions::timeout_ms(...)`：

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

當你需要不同的傳輸設定時，用 `InvoqOptions::http_client(...)` 傳入自訂的 `reqwest::Client`。`timeout_ms` 仍會控制 SDK 每次請求的逾時。

## 帳單

建立帳單：

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

沒有要設定 `.description(...)`、`.reference_id(...)` 或 `.return_url(...)` 時，直接省略即可。未設定的選填請求欄位不會出現在 JSON 裡。若要送出 `return_url: null`、不套用專案的預設 return URL，請用 `.without_return_url()`。

金額要由伺服器端決定，不要相信用戶端傳來的金額。`amount` 是 `"0.01"` 到 `"999.99"` 之間的十進位美元字串，最多兩位小數，例如 `"129"` 或 `"129.99"`。

用 `reference_id` 把 `invoice.paid` webhook 對應回你的訂單。它也讓建立動作可以放心重試：用相同的 `reference_id` 和相同的帳單條件再建立一次，回傳的是既有帳單而不是重複開單；條件不同則會回 `409 reference_id_conflict` API 錯誤。

查詢帳單：

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

`invoices.get()` 回傳結帳頁所使用的公開帳單結構。它包含面向結帳頁的欄位，例如 `amount_paid`、`amount_due`、`amount_overpaid`、`payment_status`、`project`、`deposit_address`、`monitoring_ends_at`、`monitoring_status`、`transfers` 和 `direct_onchain_rails`，但不包含 `reference_id`。需要商家端的參照時，請使用建立帳單的回應或 `invoice.paid` webhook。

建立測試付款：

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

`create_test_payment()` 只對用 `sk_test_` 金鑰建立的帳單有效。累計付款達到帳單金額時，帳單會變為 `paid`，invoq 會向你的測試 webhook URL 送出一條真實簽章的 `invoice.paid` webhook。也可以只付部分金額，這時帳單會變成 `partially_paid`。

沒有要設定 `.reference_id(...)` 時就直接省略；選填的請求字串不要送 `null`。未設定的選填字串不會出現在請求 JSON 裡。

SDK 會直接回傳回應中的 `data` 物件。

回應中的金額會經過正規化。用 `"129"` 建立，帳單會回傳 `amount: "129.0000"`。比較金額請按數值比較，不要按字串比較。`amount_due` 依 `max(amount - amount_paid, 0)` 衍生，使用和 `amount_paid` 相同的 18 位小數 scale；`amount_overpaid` 與它互為鏡像，即 `max(amount_paid - amount, 0)`，所以你不必自己做減法。`monitoring_status` 取值 `active` 或 `ended`——一旦變為 `ended`，收款位址就不再被監控——而 `transfers` 是已確認的鏈上收款紀錄（每一項都含 `tx_hash`、`amount` 和 `explorer_tx_url`）。測試帳單裡兩者分別為 `null` / `[]`。

## 託管結帳頁

每張帳單也都有一個託管結帳頁，網址是：

```text
https://pay.invoq.money/<invoice id>
```

當頁內結帳彈窗不適用時，把連結分享出去或直接導向過去即可。

## Webhook

把原始請求內容傳給 `verify_webhook`。驗證前不要先解析 JSON 再重新序列化。

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

`verify_webhook` 可以接受 `http::HeaderMap`、一個包含 `invoq-signature` 標頭值的字串，或是標頭鍵不分大小寫的其他支援的 map 型別。

webhook 驗證失敗會回傳 `InvoqSignatureVerificationError`。

用 `invoice.paid` webhook 在你的伺服器上履行訂單。`invoice_paid_event(&event)` 會為可履約的已付款事件回傳具型別的資料；如果你只需要一個布林值，`is_invoice_paid(&event)` 對同樣的事件會回傳 true。用帳單的 `reference_id` 找到並履行對應的訂單。這兩個輔助函式接受可視為已付款的帳單狀態（`paid`、`settling` 或 `settled`），並拒絕 `review_required`。`review_required` 的帳單暫時還不會發送 `invoice.paid` webhook。

投遞失敗會重試，所以請按 `reference_id` 或帳單 `id` 冪等地履行訂單，讓重複投遞不會產生任何額外效果。請盡快回應 2xx；任何其他狀態都算投遞失敗。

SDK 容許時間戳有 5 分鐘的誤差。投遞失敗時，每次重試都會重新簽章，所以正常的重試投遞仍會落在這個時間範圍內、順利通過驗證。簽章標頭是：

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## 錯誤處理

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

API 輸入驗證、連線失敗、逾時，以及回應解析失敗，都會回傳 `InvoqError`。

## 授權條款

採用 MIT 授權條款。
