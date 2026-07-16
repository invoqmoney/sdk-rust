# invoq Rust SDK'sı

[English](../README.md) · [Bahasa Indonesia](./README.id.md) · [Español](./README.es-419.md) · [Français](./README.fr.md) · [Português](./README.pt-BR.md) · [Tiếng Việt](./README.vi.md) · **Türkçe** · [ไทย](./README.th.md) · [简体中文](./README.zh-Hans.md) · [繁體中文](./README.zh-Hant.md)

> Bu belge İngilizce README'nin çevirisidir; bir fark olursa [İngilizce sürüm](../README.md) esas alınır.

invoq sunucu API'leri ve webhook doğrulaması için Rust SDK'sı.

Bu crate'i yalnızca sunucunuzda kullanın. Gizli anahtarları kabul eder ve istemci
tarafı uygulamaların içine derlenmemelidir.

## Sunucu SDK'ları

Bu dillerin herhangi biriyle arka ucunuzdan fatura oluşturun ve webhook'ları doğrulayın — aynı REST API, aynı webhook imzası. Bu repo, Rust SDK'sıdır.

| Dil | Repo |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) (`@invoq/server`) |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **bu repo** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

Hangi arka ucu seçerseniz seçin, tarayıcı tarafı aynıdır: **`@invoq/checkout`** (JavaScript, [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) içinde) her ön uç için sayfa içi ödeme penceresini açar.

## Kurulum

```toml
[dependencies]
invoq = "0.1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

SDK, `reqwest` ve async Rust kullanır.

Rust 1.86 veya üstünü gerektirir.

## Anahtarlarınızı alın

1. [invoq paneline](https://app.invoq.money) giriş yapın ve bir proje
   oluşturun.
2. **API keys** sayfasında bir gizli anahtar oluşturun. Test anahtarları `sk_test_`,
   canlı anahtarlar `sk_live_` ile başlar. Anahtarın modu, faturaların test mi
   canlı mı olacağını belirler.
3. Projenizin **webhooks** ayarlarında webhook URL'nizi kaydedin. O modun webhook
   sırrı (`whsec_...`) yalnızca bir kez, webhook'u ilk etkinleştirdiğinizde
   gösterilir — hemen saklayın. Webhook URL'leri herkese açık HTTPS URL'leri olmalı.

Gizli anahtarı sunucu ortamınıza ekleyin:

```sh
INVOQ_SECRET_KEY=sk_test_...
```

Webhook'ları işliyorsanız webhook sırrını da saklayın:

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

Test anahtarlarıyla başlayın. Canlı ortama geçerken canlı anahtara ve eşleşen
canlı webhook sırrına geçin.

## İstemci oluşturun

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

Canlı ortamda varsayılan API origin'i:

```text
https://api.invoq.money
```

Geliştirme sırasında API origin'ini değiştirin:

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

`api_origin`, yol, sorgu, hash, kullanıcı adı veya parola içermeyen mutlak bir
`http` ya da `https` origin'i olmalıdır. SDK, `/v1/...` kaynak yollarını sonuna ekler.

Varsayılan HTTP istemcisi 10 saniyelik istek zaman aşımı kullanır. Bunu değiştirmek
için `InvoqOptions::timeout_ms(...)` kullanın:

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

Farklı transport ayarlarına ihtiyaç duyduğunuzda `InvoqOptions::http_client(...)` ile
özel bir `reqwest::Client` verin. `timeout_ms` yine de istek başına SDK zaman aşımını
kontrol eder.

## Faturalar

Fatura oluşturun:

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

Ayarlı değillerse `.description(...)`, `.reference_id(...)` veya `.return_url(...)`
çağrılarını atlayın. Ayarlanmamış isteğe bağlı istek alanları JSON'dan çıkarılır.
`return_url: null` göndermek ve projenin varsayılan return URL'sini devre dışı
bırakmak için `.without_return_url()` kullanın.

Sunucu tarafında bir tutar kullanın. İstemciden gelen tutarlara güvenmeyin. `amount`,
`"0.01"` ile `"999.99"` arasında, en fazla 2 ondalık basamaklı, USD cinsinden ondalık
bir dizedir — örneğin `"129"` veya `"129.99"`.

`invoice.paid` webhook'larını siparişinize geri bağlamak için `reference_id` kullanın.
Oluşturmayı yeniden denemeyi de güvenli kılar: aynı `reference_id` ve aynı fatura
koşullarıyla tekrar oluşturursanız kopya yerine mevcut faturayı alırsınız; farklı
koşullar ise `409 reference_id_conflict` API hatasıyla başarısız olur.

Bir faturayı getirin:

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

`invoices.get()`, checkout'un kullandığı herkese açık fatura şeklini döndürür.
`amount_paid`, `amount_due`, `amount_overpaid`, `payment_status`, `project`,
`deposit_address`, `monitoring_ends_at`, `monitoring_status`, `transfers` ve
`direct_onchain_rails` gibi checkout'a yönelik alanları içerir,
ancak `reference_id` içermez. Merchant referansınız gerektiğinde oluşturma yanıtını
veya `invoice.paid` webhook'unu kullanın.

Test ödemesi oluşturun:

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

`create_test_payment()` yalnızca `sk_test_` anahtarıyla oluşturulmuş faturalarda çalışır.
Ödemeler fatura tutarına ulaştığında fatura `paid` olur ve invoq, test webhook URL'nize
gerçekten imzalanmış bir `invoice.paid` webhook'u gönderir. Kısmi tutarlara izin verilir;
sonuç `partially_paid` olur.

Ayarlı değilse `.reference_id(...)` çağrısını atlayın; isteğe bağlı istek dizeleri için
`null` göndermeyin. Ayarlanmamış isteğe bağlı dizeler istek JSON'ından çıkarılır.

SDK, yanıtın `data` nesnesini doğrudan döndürür.

Yanıtlardaki tutarlar normalize edilir. `"129"` ile oluşturun, fatura
`amount: "129.0000"` döndürür. Tutarları dize olarak değil, sayısal karşılaştırın.
`amount_due`, `max(amount - amount_paid, 0)` olarak türetilir ve `amount_paid` ile aynı
18 ondalık basamak ölçeğini kullanır; `amount_overpaid` ise onun aynasıdır,
`max(amount_paid - amount, 0)`, yani parayı kendiniz çıkarmanız hiç gerekmez.
`monitoring_status`, `active` ya da `ended` olur — `ended` olduğunda yatırma
adresi artık izlenmez — ve `transfers`, onaylanmış zincir üstü tahsilat kaydıdır
(her girdide `tx_hash`, `amount` ve `explorer_tx_url` bulunur). İkisi de test
faturaları için `null` / `[]` olur.

## Barındırılan ödeme sayfası

Her faturanın ayrıca şu adreste barındırılan bir ödeme sayfası vardır:

```text
https://pay.invoq.money/<invoice id>
```

Sayfa içi ödeme penceresi uygun olmadığında bağlantıyı paylaşın ya da oraya yönlendirin.

## Webhook'lar

Ham istek gövdesini `verify_webhook`'a verin. Doğrulamadan önce JSON'u ayrıştırıp
yeniden serileştirmeyin.

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

`verify_webhook`, bir `http::HeaderMap`, `invoq-signature` başlık değerini içeren bir
dize ya da büyük/küçük harfe duyarsız başlık anahtarlarına sahip desteklenen map türlerini
kabul eder.

Webhook doğrulama hataları `InvoqSignatureVerificationError` döndürür.

Siparişleri sunucunuzda `invoice.paid` webhook'larıyla işleyin.
`invoice_paid_event(&event)`, işlenebilir ödeme olayları için tipli veri döndürür;
yalnızca bir boolean'a ihtiyacınız olduğunda `is_invoice_paid(&event)` aynı olaylar için
true döndürür. Siparişinizi bulup işlemek için faturanın `reference_id` değerini kullanın.
Yardımcılar, ödeme tamamlanmış sayılan fatura durumlarını (`paid`, `settling` veya
`settled`) kabul eder ve `review_required` durumunu reddeder. `review_required`
durumundaki bir fatura henüz `invoice.paid` webhook'u göndermez.

Başarısız teslimatlar yeniden denenir; bu yüzden `reference_id` veya faturanın `id`'siyle
idempotent şekilde işleyin ve tekrar gelen teslimatları etkisiz kılın. Hızla 2xx dönün;
diğer her durum başarısız teslimat sayılır.

SDK, 5 dakikalık bir zaman damgası toleransı tanır. Başarısız teslimatlar her yeniden
denemede yeniden imzalanır; bu yüzden normal yeniden denenen teslimatlar yine de bu pencere
içinde doğrulanır. İmza başlığı şöyledir:

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## Hatalar

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

API girdi doğrulaması, bağlantı hataları, zaman aşımları ve yanıt ayrıştırma hataları
`InvoqError` döndürür.

## Lisans

MIT lisansı altında lisanslanmıştır.
