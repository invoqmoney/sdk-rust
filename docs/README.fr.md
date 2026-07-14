# SDK Rust invoq

[English](../README.md) · [Bahasa Indonesia](./README.id.md) · [Español](./README.es-419.md) · **Français** · [Português](./README.pt-BR.md) · [Tiếng Việt](./README.vi.md) · [Türkçe](./README.tr.md) · [ไทย](./README.th.md) · [简体中文](./README.zh-Hans.md) · [繁體中文](./README.zh-Hant.md)

> Ce document est une traduction du README anglais ; en cas de divergence, la [version anglaise](../README.md) fait foi.

SDK Rust pour les API serveur d’invoq et la vérification des webhooks.

N’utilisez ce crate que sur votre serveur. Il accepte des clés secrètes et ne doit pas être compilé dans des applications côté client.

## SDK serveur

Créez des factures et vérifiez les webhooks depuis votre backend dans l’un de ces langages — même REST API, même signature de webhook. Ce dépôt est le SDK Rust.

| Langage | Dépôt |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) (`@invoq/server`) |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **ce dépôt** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

Quel que soit le backend, le côté navigateur reste le même : **`@invoq/checkout`** (JavaScript, dans [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)) ouvre la fenêtre de paiement intégrée à la page pour n’importe quel frontend.

## Installation

```toml
[dependencies]
invoq = "0.1.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Le SDK utilise `reqwest` et du Rust asynchrone.

Nécessite Rust 1.86 ou une version plus récente.

## Récupérez vos clés

1. Connectez-vous au [tableau de bord invoq](https://app.invoq.money) et créez un projet.
2. Sur la page API keys, créez une clé secrète. Les clés de test commencent par `sk_test_`, les clés de production par `sk_live_`. Le mode de la clé détermine si les factures sont de test ou de production.
3. Dans les réglages webhooks de votre projet, enregistrez votre URL de webhook. Le secret du webhook (`whsec_...`) pour ce mode ne s’affiche qu’une seule fois, lors de la première activation du webhook — notez-le donc tout de suite. Les URL de webhook doivent être des URL HTTPS publiques.

Ajoutez la clé secrète à l’environnement de votre serveur :

```sh
INVOQ_SECRET_KEY=sk_test_...
```

Si vous gérez des webhooks, stockez aussi le secret du webhook :

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

Commencez avec les clés de test. Passez à la clé de production et au secret de webhook de production correspondant lors de la mise en production.

## Créer un client

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

L’origine par défaut de l’API en production :

```text
https://api.invoq.money
```

Remplacez l’origine de l’API en développement :

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

`api_origin` doit être une origine `http` ou `https` absolue, sans chemin, requête, fragment, nom d’utilisateur ni mot de passe. Le SDK y ajoute les chemins de ressources `/v1/...`.

Le client HTTP par défaut applique un délai d’attente de 10 secondes par requête. Utilisez `InvoqOptions::timeout_ms(...)` pour le modifier :

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

Passez un `reqwest::Client` personnalisé via `InvoqOptions::http_client(...)` lorsque vous avez besoin de réglages de transport différents. `timeout_ms` continue de contrôler le délai d’attente du SDK par requête.

## Factures

Créez une facture :

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

Omettez `.description(...)`, `.reference_id(...)` ou `.return_url(...)` lorsqu’ils ne sont pas définis. Les champs de requête optionnels non définis sont omis du JSON. Utilisez `.without_return_url()` pour envoyer `return_url: null` et renoncer à l’URL de retour par défaut du projet.

Utilisez un montant défini côté serveur. Ne faites pas confiance aux montants fournis par le client. `amount` est une chaîne décimale en USD de `"0.01"` à `"999.99"`, avec au plus 2 décimales, comme `"129"` ou `"129.99"`.

Utilisez `reference_id` pour relier les webhooks `invoice.paid` à votre commande. Il permet aussi de relancer la création sans risque : recréer une facture avec le même `reference_id` et les mêmes conditions renvoie la facture existante au lieu d’un doublon, tandis qu’avec des conditions différentes, l’appel échoue avec une erreur d’API `409 reference_id_conflict`.

Récupérez une facture :

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

`invoices.get()` renvoie la forme de facture publique utilisée par le checkout. Elle inclut les champs côté checkout, comme `amount_paid`, `amount_due`, `payment_status`, `project`, `deposit_address`, `monitoring_ends_at` et `direct_onchain_rails`, mais n’inclut pas `reference_id`. Utilisez la réponse de création ou le webhook `invoice.paid` quand vous avez besoin de votre référence marchand.

Créez un paiement de test :

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

`create_test_payment()` ne fonctionne que sur les factures créées avec une clé `sk_test_`. Quand les paiements atteignent le montant de la facture, celle-ci passe à `paid` et invoq envoie un vrai webhook `invoice.paid` signé à votre URL de webhook de test. Les montants partiels sont autorisés et produisent `partially_paid`.

Omettez `.reference_id(...)` lorsqu’il n’est pas défini ; n’envoyez pas `null` pour les chaînes de requête optionnelles. Les chaînes optionnelles non définies sont omises du JSON de la requête.

Le SDK renvoie directement l’objet `data` de la réponse.

Les montants des réponses sont normalisés. Créez avec `"129"` et la facture renvoie `amount: "129.0000"`. Comparez les montants numériquement, pas comme des chaînes. `amount_due` est dérivé sous la forme `max(amount - amount_paid, 0)` et utilise la même échelle à 18 décimales que `amount_paid`.

## Page de paiement hébergée

Chaque facture dispose aussi d’une page de paiement hébergée à l’adresse :

```text
https://pay.invoq.money/<invoice id>
```

Partagez le lien ou redirigez-y quand une fenêtre de paiement intégrée à la page ne convient pas.

## Webhooks

Transmettez le corps brut de la requête à `verify_webhook`. N’analysez pas le JSON et ne le re-sérialisez pas avant la vérification.

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

`verify_webhook` accepte un `http::HeaderMap`, une chaîne contenant la valeur de l’en-tête `invoq-signature`, ou les types de map pris en charge dont les clés d’en-tête sont insensibles à la casse.

Les échecs de vérification de webhook renvoient `InvoqSignatureVerificationError`.

Utilisez les webhooks `invoice.paid` pour traiter les commandes sur votre serveur. `invoice_paid_event(&event)` renvoie des données typées pour les événements payés permettant de traiter une commande ; `is_invoice_paid(&event)` renvoie true pour ces mêmes événements lorsque vous n’avez besoin que d’un booléen. Utilisez le `reference_id` de la facture pour retrouver et traiter votre commande. Les helpers acceptent les statuts de facture assimilables à un paiement validé (`paid`, `settling` ou `settled`) et rejettent `review_required`. Une facture `review_required` n’émet pas encore de webhook `invoice.paid`.

Les livraisons échouées sont retentées : traitez donc les commandes de façon idempotente par `reference_id` ou par `id` de facture, et faites en sorte qu’une livraison répétée n’ait aucun effet. Répondez rapidement avec un 2xx ; tout autre statut compte comme une livraison échouée.

Le SDK tolère un décalage d’horodatage de 5 minutes. Les livraisons échouées sont signées à nouveau à chaque tentative, si bien que les livraisons normalement retentées se vérifient toujours dans cette fenêtre. L’en-tête de signature est :

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## Erreurs

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

Les erreurs de validation des entrées de l’API, les échecs de connexion, les délais d’attente dépassés et les échecs d’analyse des réponses renvoient `InvoqError`.

## Licence

Distribué sous licence MIT.
