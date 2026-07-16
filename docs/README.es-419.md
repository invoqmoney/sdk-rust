# SDK de invoq para Rust

[English](../README.md) · [Bahasa Indonesia](./README.id.md) · **Español** · [Français](./README.fr.md) · [Português](./README.pt-BR.md) · [Tiếng Việt](./README.vi.md) · [Türkçe](./README.tr.md) · [ไทย](./README.th.md) · [简体中文](./README.zh-Hans.md) · [繁體中文](./README.zh-Hant.md)

> Este documento es una traducción del README en inglés; si algo difiere, vale la [versión en inglés](../README.md).

SDK de Rust para las APIs de servidor de invoq y la verificación de webhooks.

Usa este crate solo en tu servidor. Acepta claves secretas y no debe compilarse
en aplicaciones del lado del cliente.

## SDKs de servidor

Crea facturas y verifica webhooks desde tu backend en cualquiera de estos lenguajes — la misma REST API y la misma firma de webhook. Este repositorio es el SDK de Rust.

| Lenguaje | Repositorio |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) (`@invoq/server`) |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **este repositorio** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

El lado del navegador es el mismo para cualquier backend: **`@invoq/checkout`** (JavaScript, en [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)) abre la ventana de pago integrada en la página para cualquier frontend.

## Instalación

```toml
[dependencies]
invoq = "0.2.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

El SDK usa `reqwest` y Rust asíncrono.

Requiere Rust 1.86 o más nuevo.

## Consigue tus claves

1. Inicia sesión en el [panel de invoq](https://app.invoq.money) y crea un
   proyecto.
2. En la página **API keys**, crea una clave secreta. Las claves de prueba
   empiezan con `sk_test_`, las claves de producción con `sk_live_`. El modo de
   la clave determina si las facturas son de prueba o de producción.
3. En la configuración de **webhooks** de tu proyecto, guarda tu URL de webhook.
   El secreto del webhook (`whsec_...`) de ese modo se muestra una sola vez,
   cuando activas el webhook por primera vez, así que guárdalo de inmediato. Las
   URL de webhook deben ser HTTPS y públicas.

Agrega la clave secreta al entorno de tu servidor:

```sh
INVOQ_SECRET_KEY=sk_test_...
```

Si manejas webhooks, guarda también el secreto del webhook:

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

Empieza con las claves de prueba. Cambia a la clave de producción y al secreto de
webhook de producción correspondiente cuando pases a producción.

## Crea un cliente

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

Origin de la API predeterminado en producción:

```text
https://api.invoq.money
```

Sobrescribe el origin de la API durante el desarrollo:

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

`api_origin` debe ser un origin `http` o `https` absoluto, sin ruta, query, hash,
usuario ni contraseña. El SDK agrega las rutas de recursos `/v1/...`.

El cliente HTTP predeterminado usa un tiempo de espera de 10 segundos por
solicitud. Usa `InvoqOptions::timeout_ms(...)` para cambiarlo:

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

Pasa un `reqwest::Client` personalizado con `InvoqOptions::http_client(...)` cuando
necesites otra configuración de transporte. `timeout_ms` sigue controlando el
tiempo de espera del SDK por solicitud.

## Facturas

Crea una factura:

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

Omite `.description(...)`, `.reference_id(...)` o `.return_url(...)` cuando no los
definas. Los campos opcionales de la solicitud sin definir se omiten del JSON. Usa
`.without_return_url()` para enviar `return_url: null` y no usar la URL de retorno
predeterminada del proyecto.

Usa un monto definido en el servidor. No confíes en los montos que manda el
cliente. `amount` es una cadena decimal en USD de `"0.01"` a `"999.99"` con hasta 2
decimales, como `"129"` o `"129.99"`.

Usa `reference_id` para vincular los webhooks `invoice.paid` con tu pedido. También
hace que puedas reintentar la creación sin riesgo: si vuelves a crear la factura con
el mismo `reference_id` y los mismos términos, recibes la factura existente en lugar
de un duplicado, mientras que con términos distintos falla con un error de API
`409 reference_id_conflict`.

Obtén una factura:

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

`invoices.get()` devuelve la forma de factura pública usada por el checkout. Incluye
campos orientados al checkout como `amount_paid`, `amount_due`, `amount_overpaid`,
`payment_status`, `project`, `deposit_address`, `monitoring_ends_at`,
`monitoring_status`, `transfers` y `direct_onchain_rails`, pero no
incluye `reference_id`. Usa la respuesta de creación o el webhook `invoice.paid`
cuando necesites tu referencia de comercio.

Crea un pago de prueba:

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

`create_test_payment()` solo funciona con facturas creadas con una clave `sk_test_`.
Cuando los pagos alcanzan el monto de la factura, la factura pasa a `paid` e invoq
envía un webhook `invoice.paid` firmado de verdad a tu URL de webhook de prueba. Se
permiten montos parciales, que producen `partially_paid`.

Omite `.reference_id(...)` cuando no lo definas; no envíes `null` para las cadenas
opcionales de la solicitud. Las cadenas opcionales sin definir se omiten del JSON de
la solicitud.

El SDK devuelve directamente el objeto `data` de la respuesta.

Los montos en las respuestas se normalizan. Crea con `"129"` y la factura devuelve
`amount: "129.0000"`. Compara los montos numéricamente, no como cadenas. `amount_due`
se deriva como `max(amount - amount_paid, 0)` y usa la misma escala de 18 decimales
que `amount_paid`; `amount_overpaid` es su reflejo, `max(amount_paid - amount, 0)`,
así que nunca restas dinero por tu cuenta. `monitoring_status` es `active` o
`ended` — una vez que es `ended`, la dirección de depósito deja de vigilarse — y
`transfers` es el registro confirmado de recepciones on-chain (cada entrada tiene
`tx_hash`, `amount` y `explorer_tx_url`). Ambos son `null` / `[]` en las facturas
de prueba.

## Página de pago alojada

Cada factura también tiene una página de pago alojada en:

```text
https://pay.invoq.money/<invoice id>
```

Comparte el enlace o redirige ahí cuando una ventana de pago integrada en la página
no encaje.

## Webhooks

Pasa el cuerpo sin procesar de la solicitud a `verify_webhook`. No proceses el JSON
ni lo vuelvas a serializar antes de la verificación.

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

`verify_webhook` acepta un `http::HeaderMap`, una cadena con el valor del encabezado
`invoq-signature`, o tipos de mapa compatibles con claves de encabezado que no
distinguen mayúsculas de minúsculas.

Las fallas de verificación de webhook devuelven `InvoqSignatureVerificationError`.

Usa los webhooks `invoice.paid` para procesar los pedidos en tu servidor.
`invoice_paid_event(&event)` devuelve datos tipados para los eventos de pago que
puedes procesar; `is_invoice_paid(&event)` devuelve true para esos mismos eventos
cuando solo necesitas un booleano. Usa el `reference_id` de la factura para encontrar
y procesar tu pedido. Los helpers aceptan estados de factura equivalentes a pagada
(`paid`, `settling` o `settled`) y rechazan `review_required`. Una factura
`review_required` aún no emite un webhook `invoice.paid`.

Las entregas fallidas se reintentan, así que procesa de forma idempotente por
`reference_id` o por el `id` de la factura y trata las entregas repetidas como una
operación sin efecto. Responde con un 2xx rápido; cualquier otro estado cuenta como
entrega fallida.

El SDK permite una tolerancia de 5 minutos en el timestamp. Las entregas fallidas se
firman de nuevo en cada reintento, así que las entregas reintentadas normales siguen
pasando la verificación dentro de esa ventana. El encabezado de firma es:

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## Errores

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

La validación de entrada de la API, las fallas de conexión, los tiempos de espera
agotados y las fallas al procesar la respuesta devuelven `InvoqError`.

## Licencia

Con licencia MIT.
