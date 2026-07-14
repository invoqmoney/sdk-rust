# invoq Rust SDK

[English](../README.md) · [Bahasa Indonesia](./README.id.md) · [Español](./README.es-419.md) · [Français](./README.fr.md) · [Português](./README.pt-BR.md) · [Tiếng Việt](./README.vi.md) · [Türkçe](./README.tr.md) · **ไทย** · [简体中文](./README.zh-Hans.md) · [繁體中文](./README.zh-Hant.md)

> เอกสารนี้แปลจาก README ภาษาอังกฤษ หากมีข้อความไม่ตรงกัน ให้ยึด[ฉบับภาษาอังกฤษ](../README.md)เป็นหลัก

Rust SDK สำหรับ server API ของ invoq และการตรวจสอบ webhook

ใช้ crate นี้บนเซิร์ฟเวอร์ของคุณเท่านั้น เพราะรับคีย์ลับ (secret key) จึงต้องไม่ถูกคอมไพล์เข้าไปในแอปพลิเคชันฝั่งไคลเอนต์

## SDK ฝั่งเซิร์ฟเวอร์

สร้างใบแจ้งหนี้และตรวจสอบ webhook จากแบ็กเอนด์ของคุณด้วยภาษาใดก็ได้เหล่านี้ — REST API และลายเซ็น webhook เหมือนกันทุกภาษา repo นี้คือ SDK สำหรับ Rust

| ภาษา | Repo |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) (`@invoq/server`) |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **repo นี้** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

จะเลือกแบ็กเอนด์ตัวไหนก็ตาม ฝั่งเบราว์เซอร์เหมือนกันหมด: **`@invoq/checkout`** (JavaScript อยู่ใน [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)) เปิดหน้าชำระเงินแบบฝังในหน้าเว็บให้ฟรอนต์เอนด์ใดก็ได้

## ติดตั้ง

```toml
[dependencies]
invoq = "0.1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

SDK นี้ใช้ `reqwest` และ async Rust

ต้องใช้ Rust 1.86 ขึ้นไป

## รับคีย์ของคุณ

1. เข้าสู่ระบบ[แดชบอร์ด invoq](https://app.invoq.money) แล้วสร้างโปรเจกต์
2. ที่หน้า **API keys** ให้สร้างคีย์ลับ (secret key) ขึ้นมา คีย์ทดสอบขึ้นต้นด้วย `sk_test_` คีย์จริงขึ้นต้นด้วย `sk_live_` โหมดของคีย์เป็นตัวกำหนดว่าใบแจ้งหนี้ที่สร้างจะเป็นแบบทดสอบหรือของจริง
3. ในการตั้งค่า **webhooks** ของโปรเจกต์ ให้บันทึก URL ของ webhook ที่จะใช้ ซีเคร็ตของ webhook (`whsec_...`) สำหรับโหมดนั้นจะแสดงแค่ครั้งเดียวตอนเปิดใช้ webhook ครั้งแรก จึงควรรีบเก็บไว้ทันที URL ของ webhook ต้องเป็น HTTPS ที่เข้าถึงได้แบบสาธารณะ

เพิ่มคีย์ลับเข้าเป็นตัวแปรสภาพแวดล้อมของเซิร์ฟเวอร์:

```sh
INVOQ_SECRET_KEY=sk_test_...
```

ถ้าคุณต้องจัดการ webhook ด้วย ให้เก็บซีเคร็ตของ webhook ไว้ด้วย:

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

เริ่มจากคีย์ทดสอบก่อน แล้วค่อยสลับไปใช้คีย์จริงกับซีเคร็ต webhook ของจริงที่คู่กันตอนขึ้นใช้งานจริง

## สร้างไคลเอนต์

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

ค่าเริ่มต้นของ API origin ในสภาพแวดล้อมจริง:

```text
https://api.invoq.money
```

ปรับทับค่า API origin ระหว่างการพัฒนา:

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

`api_origin` ต้องเป็น origin แบบ `http` หรือ `https` เต็มรูปแบบ โดยไม่มีพาธ, query, hash, ชื่อผู้ใช้ หรือรหัสผ่าน SDK จะต่อท้ายด้วยพาธทรัพยากร `/v1/...` ให้เอง

ไคลเอนต์ HTTP เริ่มต้นใช้ timeout ของ request ที่ 10 วินาที ใช้ `InvoqOptions::timeout_ms(...)` เพื่อเปลี่ยนค่านี้ได้:

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

ส่ง `reqwest::Client` ที่กำหนดเองผ่าน `InvoqOptions::http_client(...)` ได้เมื่อคุณต้องการตั้งค่า transport แบบอื่น ทั้งนี้ `timeout_ms` ยังคงควบคุม timeout ต่อหนึ่ง request ของ SDK อยู่เหมือนเดิม

## ใบแจ้งหนี้

สร้างใบแจ้งหนี้:

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

ละ `.description(...)`, `.reference_id(...)` หรือ `.return_url(...)` ไว้ได้เมื่อไม่ได้กำหนดค่า ฟิลด์ optional ใน request ที่ไม่ได้กำหนดค่าจะไม่ถูกใส่ลงใน JSON ใช้ `.without_return_url()` เพื่อส่ง `return_url: null` และเลือกไม่ใช้ return URL เริ่มต้นของโปรเจกต์

ใช้ยอดเงินที่กำหนดจากฝั่งเซิร์ฟเวอร์ อย่าเชื่อยอดเงินที่ส่งมาจากฝั่งไคลเอนต์ `amount` เป็นสตริงเลขทศนิยมสกุล USD ตั้งแต่ `"0.01"` ถึง `"999.99"` ทศนิยมไม่เกิน 2 ตำแหน่ง เช่น `"129"` หรือ `"129.99"`

ใช้ `reference_id` เพื่อโยง webhook `invoice.paid` กลับไปหาคำสั่งซื้อของคุณ และยังทำให้การสร้างใบแจ้งหนี้ลองใหม่ได้อย่างปลอดภัยด้วย: ถ้าสร้างซ้ำด้วย `reference_id` เดิมและเงื่อนไขใบแจ้งหนี้เดิม จะได้ใบแจ้งหนี้ใบเดิมกลับมาแทนที่จะเกิดใบซ้ำ ส่วนเงื่อนไขที่ต่างออกไปจะล้มเหลวด้วยข้อผิดพลาด API `409 reference_id_conflict`

ดึงข้อมูลใบแจ้งหนี้:

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

`invoices.get()` จะคืนรูปแบบใบแจ้งหนี้สาธารณะที่ checkout ใช้ โดยมีฟิลด์ฝั่ง checkout เช่น `amount_paid`, `amount_due`, `payment_status`, `project`, `deposit_address`, `monitoring_ends_at` และ `direct_onchain_rails` แต่ไม่มี `reference_id` เมื่อคุณต้องใช้ reference ฝั่ง merchant ให้ใช้ response ตอนสร้างใบแจ้งหนี้หรือ webhook `invoice.paid`

สร้างการชำระเงินทดสอบ:

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

`create_test_payment()` ใช้ได้เฉพาะกับใบแจ้งหนี้ที่สร้างด้วยคีย์ `sk_test_` เท่านั้น เมื่อยอดชำระครบตามจำนวนของใบแจ้งหนี้ ใบแจ้งหนี้จะกลายเป็น `paid` แล้ว invoq จะส่ง webhook `invoice.paid` ที่ลงลายเซ็นจริงไปยัง URL webhook ทดสอบของคุณ จ่ายบางส่วนก็ได้ ผลจะเป็น `partially_paid`

ละ `.reference_id(...)` ไว้ได้เมื่อไม่ได้กำหนดค่า อย่าส่ง `null` สำหรับสตริง optional ใน request สตริง optional ที่ไม่ได้กำหนดค่าจะไม่ถูกใส่ลงใน JSON ของ request

SDK จะคืนออบเจกต์ `data` ของ response กลับมาโดยตรง

ยอดเงินในการตอบกลับจะถูกปรับให้อยู่ในรูปแบบมาตรฐาน (normalized) สร้างด้วย `"129"` แล้วใบแจ้งหนี้จะตอบกลับ `amount: "129.0000"` เวลาเทียบยอดเงินให้เทียบเป็นตัวเลข อย่าเทียบเป็นสตริง `amount_due` คำนวณจาก `max(amount - amount_paid, 0)` และใช้สเกลทศนิยม 18 ตำแหน่งเหมือน `amount_paid`

## หน้าชำระเงินที่โฮสต์ให้

ใบแจ้งหนี้ทุกใบยังมีหน้าชำระเงินที่โฮสต์ให้อยู่แล้วที่:

```text
https://pay.invoq.money/<invoice id>
```

แชร์ลิงก์หรือเปลี่ยนเส้นทางไปที่หน้านั้นได้เลยเมื่อหน้าชำระเงินแบบฝังในหน้าเว็บไม่ตอบโจทย์

## Webhooks

ส่งเนื้อหา request ดิบเข้าไปที่ `verify_webhook` อย่า parse JSON แล้ว serialize กลับใหม่ก่อนการตรวจสอบ

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

`verify_webhook` รับได้ทั้ง `http::HeaderMap`, สตริงที่มีค่าของเฮดเดอร์ `invoq-signature`, หรือชนิด map ที่รองรับซึ่งใช้คีย์เฮดเดอร์แบบไม่สนตัวพิมพ์เล็ก-ใหญ่

เมื่อการตรวจสอบ webhook ล้มเหลว จะคืน `InvoqSignatureVerificationError`

ใช้ webhook `invoice.paid` ในการจัดการคำสั่งซื้อบนเซิร์ฟเวอร์ของคุณ `invoice_paid_event(&event)` จะคืนข้อมูลแบบมีชนิด (typed) สำหรับเหตุการณ์ชำระเงินที่พร้อมจัดการคำสั่งซื้อ ส่วน `is_invoice_paid(&event)` จะคืนค่า true สำหรับเหตุการณ์เดียวกัน เหมาะเมื่อคุณต้องการแค่ค่า boolean ใช้ `reference_id` ของใบแจ้งหนี้ไปหาและจัดการคำสั่งซื้อของคุณ helper เหล่านี้รับสถานะใบแจ้งหนี้ที่ถือว่าชำระแล้ว (`paid`, `settling` หรือ `settled`) และปฏิเสธ `review_required` ใบแจ้งหนี้ที่เป็น `review_required` จะยังไม่ส่ง webhook `invoice.paid`

การส่งที่ล้มเหลวจะถูกส่งซ้ำ ดังนั้นให้จัดการคำสั่งซื้อแบบ idempotent โดยอิงจาก `reference_id` หรือ `id` ของใบแจ้งหนี้ และทำให้การส่งซ้ำเป็นการทำงานที่ไม่เกิดผลอะไร (no-op) ตอบกลับด้วย 2xx ให้เร็ว สถานะอื่นใดถือว่าเป็นการส่งที่ล้มเหลว

SDK ยอมให้ timestamp คลาดเคลื่อนได้ 5 นาที การส่งที่ล้มเหลวจะถูกลงลายเซ็นใหม่ทุกครั้งที่ส่งซ้ำ การส่งซ้ำตามปกติจึงยังตรวจสอบผ่านได้ภายในกรอบเวลานี้ เฮดเดอร์ลายเซ็นมีรูปแบบดังนี้:

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## ข้อผิดพลาด

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

การตรวจสอบความถูกต้องของอินพุต API, การเชื่อมต่อล้มเหลว, การหมดเวลา (timeout) และการ parse response ล้มเหลว จะคืน `InvoqError`

## สัญญาอนุญาต

เผยแพร่ภายใต้สัญญาอนุญาต MIT
