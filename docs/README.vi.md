# invoq Rust SDK

[English](../README.md) · [Bahasa Indonesia](./README.id.md) · [Español](./README.es-419.md) · [Français](./README.fr.md) · [Português](./README.pt-BR.md) · **Tiếng Việt** · [Türkçe](./README.tr.md) · [ไทย](./README.th.md) · [简体中文](./README.zh-Hans.md) · [繁體中文](./README.zh-Hant.md)

> Tài liệu này được dịch từ README tiếng Anh; nếu có chỗ khác nhau, [bản tiếng Anh](../README.md) là bản chuẩn.

SDK Rust cho các API máy chủ của invoq và xác minh webhook.

Chỉ dùng crate này trên máy chủ của bạn. Nó nhận các khóa bí mật và không được
biên dịch vào các ứng dụng phía client.

## SDK server

Tạo hóa đơn và xác minh webhook từ backend của bạn bằng bất kỳ ngôn ngữ nào dưới đây — cùng REST API, cùng chữ ký webhook. Repo này là SDK Rust.

| Ngôn ngữ | Repo |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) (`@invoq/server`) |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **repo này** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

Dù bạn chọn backend nào, phía trình duyệt vẫn như nhau: **`@invoq/checkout`** (JavaScript, trong [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)) mở cửa sổ thanh toán nhúng trong trang cho mọi frontend.

## Cài đặt

```toml
[dependencies]
invoq = "0.1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

SDK dùng `reqwest` và Rust bất đồng bộ.

Yêu cầu Rust 1.86 trở lên.

## Lấy khóa API

1. Đăng nhập [bảng điều khiển invoq](https://app.invoq.money) và tạo một
   dự án.
2. Ở trang **API keys**, tạo một khóa bí mật. Khóa thử nghiệm bắt đầu bằng `sk_test_`,
   khóa thật bằng `sk_live_`. Loại khóa quyết định hóa đơn là thử nghiệm hay
   thật.
3. Trong phần cài đặt **webhooks** của dự án, lưu URL webhook của bạn. Mã bí mật
   webhook (`whsec_...`) cho chế độ đó chỉ hiện đúng một lần, lúc bạn bật webhook
   lần đầu — hãy lưu lại ngay. URL webhook phải là URL HTTPS truy cập công khai được.

Thêm khóa bí mật vào biến môi trường của máy chủ:

```sh
INVOQ_SECRET_KEY=sk_test_...
```

Nếu bạn xử lý webhook, hãy lưu thêm mã bí mật webhook:

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

Bắt đầu bằng khóa thử nghiệm. Khi chạy thật thì đổi sang khóa thật và mã bí mật
webhook thật tương ứng.

## Tạo client

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

Origin API mặc định khi chạy thật:

```text
https://api.invoq.money
```

Ghi đè origin API khi phát triển:

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

`api_origin` phải là một origin `http` hoặc `https` tuyệt đối, không có path, query,
hash, tên đăng nhập hay mật khẩu. SDK sẽ nối thêm các đường dẫn tài nguyên `/v1/...`.

HTTP client mặc định dùng thời gian chờ request là 10 giây. Dùng
`InvoqOptions::timeout_ms(...)` để thay đổi:

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

Truyền một `reqwest::Client` tùy chỉnh qua `InvoqOptions::http_client(...)` khi bạn
cần các thiết lập transport khác. `timeout_ms` vẫn kiểm soát thời gian chờ mỗi
request của SDK.

## Hóa đơn

Tạo một hóa đơn:

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

Bỏ qua `.description(...)`, `.reference_id(...)` hay `.return_url(...)` khi chúng
không được đặt. Các trường tùy chọn không được đặt sẽ bị lược khỏi JSON. Dùng
`.without_return_url()` để gửi `return_url: null` và không dùng return URL
mặc định của dự án.

Hãy dùng số tiền do máy chủ quyết định. Đừng tin số tiền do phía client gửi lên.
`amount` là chuỗi thập phân USD từ `"0.01"` đến `"999.99"`, tối đa 2 chữ số lẻ,
ví dụ `"129"` hoặc `"129.99"`.

Dùng `reference_id` để nối các webhook `invoice.paid` về đúng đơn hàng của bạn. Nó
cũng giúp thao tác tạo an toàn khi thử lại: tạo lại với cùng `reference_id` và
cùng nội dung hóa đơn sẽ trả về hóa đơn đã có thay vì tạo trùng; còn nếu nội dung
khác nhau, API sẽ báo lỗi `409 reference_id_conflict`.

Lấy một hóa đơn:

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

`invoices.get()` trả về dạng hóa đơn công khai mà checkout sử dụng. Nó bao gồm
các trường dành cho checkout như `amount_paid`, `amount_due`, `amount_overpaid`,
`payment_status`, `project`, `deposit_address`, `monitoring_ends_at`,
`monitoring_status`, `transfers` và `direct_onchain_rails`,
nhưng không bao gồm `reference_id`. Hãy dùng phản hồi khi tạo hóa đơn hoặc webhook
`invoice.paid` khi bạn cần mã tham chiếu phía merchant của mình.

Tạo một khoản thanh toán thử nghiệm:

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

`create_test_payment()` chỉ dùng được với hóa đơn được tạo bằng khóa `sk_test_`.
Khi số tiền thanh toán đạt đủ giá trị hóa đơn, hóa đơn chuyển sang `paid` và invoq
gửi một webhook `invoice.paid` có chữ ký thật đến URL webhook thử nghiệm của bạn.
Cho phép các khoản trả từng phần, và chúng tạo ra `partially_paid`.

Bỏ qua `.reference_id(...)` khi không đặt; đừng gửi `null` cho các chuỗi tùy chọn
của request. Các chuỗi tùy chọn không được đặt sẽ bị lược khỏi JSON của request.

SDK trả về trực tiếp đối tượng `data` của phản hồi.

Số tiền trong phản hồi được chuẩn hóa. Tạo với `"129"` thì hóa đơn trả về
`amount: "129.0000"`. Hãy so sánh số tiền theo giá trị số, đừng so sánh dạng
chuỗi. `amount_due` được tính là `max(amount - amount_paid, 0)` và dùng cùng
thang 18 chữ số thập phân như `amount_paid`; `amount_overpaid` là bản đối xứng của
nó, `max(amount_paid - amount, 0)`, nên bạn không bao giờ phải tự trừ tiền.
`monitoring_status` là `active` hoặc `ended` — khi đã là `ended`, địa chỉ nạp tiền
không còn được theo dõi nữa — còn `transfers` là danh sách biên nhận trên chuỗi đã
xác nhận (mỗi mục có `tx_hash`, `amount` và `explorer_tx_url`). Cả hai đều là
`null` / `[]` với hóa đơn thử nghiệm.

## Trang thanh toán được lưu trữ sẵn

Mỗi hóa đơn còn có một trang thanh toán được lưu trữ sẵn tại:

```text
https://pay.invoq.money/<invoice id>
```

Cứ gửi link hoặc chuyển hướng sang đó khi cửa sổ thanh toán nhúng trong trang không phù hợp.

## Webhook

Truyền nội dung request gốc cho `verify_webhook`. Đừng phân tích rồi tuần tự hóa
lại JSON trước khi xác minh.

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

`verify_webhook` chấp nhận một `http::HeaderMap`, một chuỗi chứa giá trị của header
`invoq-signature`, hoặc các kiểu map được hỗ trợ với khóa header không phân biệt
hoa thường.

Việc xác minh webhook thất bại sẽ trả về `InvoqSignatureVerificationError`.

Dùng webhook `invoice.paid` để xử lý đơn hàng trên máy chủ của bạn.
`invoice_paid_event(&event)` trả về dữ liệu có kiểu cho các sự kiện đã thanh toán
có thể xử lý; `is_invoice_paid(&event)` trả về true cho cùng các sự kiện đó khi bạn
chỉ cần một giá trị boolean. Dùng `reference_id` của hóa đơn để tìm và xử lý đơn
hàng của bạn. Các hàm trợ giúp chấp nhận các trạng thái hóa đơn tương đương đã
thanh toán (`paid`, `settling` hoặc `settled`) và từ chối `review_required`.
Hóa đơn ở trạng thái `review_required` hiện chưa gửi webhook `invoice.paid`.

Các lần gửi thất bại sẽ được gửi lại, nên hãy xử lý đơn hàng một cách idempotent
dựa trên `reference_id` hoặc `id` hóa đơn và để các lần gửi lặp lại không gây thêm
tác dụng nào. Hãy trả về 2xx thật nhanh; mọi mã trạng thái khác đều bị tính là gửi thất bại.

SDK cho phép sai lệch timestamp tối đa 5 phút. Các lần gửi thất bại được ký lại ở
mỗi lần thử lại, nên các lần gửi lại thông thường vẫn xác minh được trong khoảng
thời gian đó. Header chữ ký có dạng:

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## Lỗi

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

Lỗi kiểm tra tính hợp lệ đầu vào của API, lỗi kết nối, hết thời gian chờ và lỗi
phân tích phản hồi đều trả về `InvoqError`.

## Giấy phép

Được cấp phép theo giấy phép MIT.
