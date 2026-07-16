# invoq Rust SDK

**English** · [Bahasa Indonesia](./docs/README.id.md) · [Español](./docs/README.es-419.md) · [Français](./docs/README.fr.md) · [Português](./docs/README.pt-BR.md) · [Tiếng Việt](./docs/README.vi.md) · [Türkçe](./docs/README.tr.md) · [ไทย](./docs/README.th.md) · [简体中文](./docs/README.zh-Hans.md) · [繁體中文](./docs/README.zh-Hant.md)

Rust SDK for invoq server APIs and webhook verification.

Use this crate only on your server. It accepts secret keys and must not be
compiled into client-side applications.

## Server SDKs

Create invoices and verify webhooks from your backend in any of these languages — same REST API, same webhook signature. This repository is the Rust SDK.

| Language | Repository |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) (`@invoq/server`) |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **this repo** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

The browser side is the same for every backend: **`@invoq/checkout`** (JavaScript, in [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)) opens the in-page checkout modal for any frontend.

## Installation

```toml
[dependencies]
invoq = "0.2.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The SDK uses `reqwest` and async Rust.

Requires Rust 1.86 or newer.

## Get your keys

1. Sign in to the [invoq dashboard](https://app.invoq.money) and create a
   project.
2. On the API keys page, create a secret key. Test keys start with `sk_test_`,
   live keys with `sk_live_`. The key mode determines whether invoices are test
   or live.
3. In your project's webhooks settings, save your webhook URL. The webhook
   secret (`whsec_...`) for that mode is shown once when you first enable the
   webhook, so store it right away. Webhook URLs must be public HTTPS URLs.

Add the secret key to your server environment:

```sh
INVOQ_SECRET_KEY=sk_test_...
```

If you handle webhooks, also store the webhook secret:

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

Start with test keys. Switch to the live key and matching live webhook secret
when you go to production.

## Create a client

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

Production API origin default:

```text
https://api.invoq.money
```

Override the API origin during development:

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

`api_origin` must be an absolute `http` or `https` origin without a path, query,
hash, username, or password. The SDK appends `/v1/...` resource paths.

The default HTTP client uses a 10 second request timeout. Use
`InvoqOptions::timeout_ms(...)` to change it:

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

Pass a custom `reqwest::Client` with `InvoqOptions::http_client(...)` when you
need different transport settings. `timeout_ms` still controls the per-request
SDK timeout.

## Invoices

Create an invoice:

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

Omit `.description(...)`, `.reference_id(...)`, or `.return_url(...)` when they
are not set. Unset optional request fields are omitted from JSON. Use
`.without_return_url()` to send `return_url: null` and opt out of a project
default return URL.

Use a server-side amount. Do not trust client-supplied amounts. `amount` is a
decimal USD string from `"0.01"` to `"999.99"` with up to 2 decimal places, such
as `"129"` or `"129.99"`.

Use `reference_id` to map `invoice.paid` webhooks back to your order. It also
makes creation retry-safe: creating again with the same `reference_id` and the
same invoice terms returns the existing invoice instead of a duplicate, while
different terms fail with a `409 reference_id_conflict` API error.

Get an invoice:

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

`invoices.get()` returns the public invoice shape used by checkout. It includes
checkout-facing fields such as `amount_paid`, `amount_due`, `amount_overpaid`,
`payment_status`, `project`, `deposit_address`, `monitoring_ends_at`,
`monitoring_status`, `transfers`, and `direct_onchain_rails`,
but it does not include `reference_id`. Use the create response or the
`invoice.paid` webhook when you need your merchant reference.

Create a test payment:

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

`create_test_payment()` only works on invoices created with a `sk_test_` key.
When payments reach the invoice amount, the invoice becomes `paid` and invoq
sends a real signed `invoice.paid` webhook to your test webhook URL. Partial
amounts are allowed and produce `partially_paid`.

Omit `.reference_id(...)` when it is not set; do not send `null` for optional
request strings. Unset optional strings are omitted from request JSON.

The SDK returns the response `data` object directly.

Amounts in responses are normalized. Create with `"129"` and the invoice returns
`amount: "129.0000"`. Compare amounts numerically, not as strings. `amount_due`
is derived as `max(amount - amount_paid, 0)` and uses the same 18-decimal scale
as `amount_paid`; `amount_overpaid` is its mirror, `max(amount_paid - amount, 0)`,
so you never subtract money yourself. `monitoring_status` is `active` or `ended`
— once it is `ended`, the deposit address is no longer watched — and `transfers`
is the confirmed on-chain receipt trail (each entry has `tx_hash`, `amount`, and
`explorer_tx_url`). Both are `null` / `[]` for test invoices.

## Hosted checkout page

Every invoice also has a hosted checkout page at:

```text
https://pay.invoq.money/<invoice id>
```

Share the link or redirect to it when an in-page checkout modal is not a fit.

## Webhooks

Pass the raw request body to `verify_webhook`. Do not parse JSON and re-serialize
it before verification.

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

`verify_webhook` accepts an `http::HeaderMap`, a string containing the
`invoq-signature` header value, or supported map types with case-insensitive
header keys.

Webhook verification failures return `InvoqSignatureVerificationError`.

Use `invoice.paid` webhooks to fulfill orders on your server.
`invoice_paid_event(&event)` returns typed data for fulfillable paid events;
`is_invoice_paid(&event)` returns true for the same events when you only need a
boolean. Use the invoice `reference_id` to find and fulfill your order. The
helpers accept paid-equivalent invoice statuses (`paid`, `settling`, or
`settled`) and reject `review_required`. A `review_required` invoice does not
emit an `invoice.paid` webhook yet.

Failed deliveries are retried, so fulfill idempotently by `reference_id` or
invoice `id` and make repeat deliveries a no-op. Respond with a 2xx quickly; any
other status counts as a failed delivery.

The SDK allows a 5-minute timestamp tolerance. Failed deliveries are signed again
on each retry, so normal retried deliveries still verify inside that window. The
signature header is:

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## Errors

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

API input validation, connection failures, timeouts, and response parse failures
return `InvoqError`.

## License

Licensed under the MIT license.
