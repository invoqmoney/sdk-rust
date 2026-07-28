# SDK Rust da invoq

[English](https://github.com/invoqmoney/sdk-rust/blob/main/README.md) · [Bahasa Indonesia](./README.id.md) · [Español](./README.es-419.md) · [Français](./README.fr.md) · **Português** · [Tiếng Việt](./README.vi.md) · [Türkçe](./README.tr.md) · [ไทย](./README.th.md) · [简体中文](./README.zh-Hans.md) · [繁體中文](./README.zh-Hant.md)

> Este documento é uma tradução do README em inglês; se algo divergir, vale a [versão em inglês](https://github.com/invoqmoney/sdk-rust/blob/main/README.md).

SDK Rust para as APIs de servidor da invoq e a verificação de webhooks.

Use este crate apenas no seu servidor. Ele aceita chaves secretas e não deve ser
compilado em aplicações do lado do cliente.

**Programa com IA? Cole isto.**

```text
Adicione pagamentos em stablecoin ao meu projeto com invoq. Comece no modo de teste. Leia a documentação antes de escrever código: https://invoq.money/llms.txt
```

## SDKs de servidor

Crie faturas e verifique webhooks a partir do seu backend em qualquer uma destas linguagens — mesma REST API, mesma assinatura de webhook. Este repositório é o SDK de Rust.

| Linguagem | Repositório |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) (`@invoq/server`) |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **este repositório** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

O lado do navegador é o mesmo para todo backend: **`@invoq/checkout`** (JavaScript, em [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)) abre a janela de checkout dentro da página para qualquer frontend.

## Instalação

```toml
[dependencies]
invoq = "0.3.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

O SDK usa `reqwest` e Rust assíncrono.

Requer Rust 1.86 ou mais novo.

## Pegue suas chaves

1. Entre no [painel da invoq](https://app.invoq.money) e crie um
   projeto.
2. Na página **API keys**, crie uma chave secreta. Chaves de teste começam com `sk_test_`,
   chaves de produção com `sk_live_`. O modo da chave define se as faturas são de teste
   ou de produção.
3. Nas configurações de **webhooks** do projeto, salve a URL do seu webhook. O segredo do webhook
   (`whsec_...`) daquele modo aparece uma única vez, quando você ativa o webhook
   pela primeira vez — então guarde na hora. As URLs de webhook precisam ser URLs HTTPS públicas.
4. Configure a sua **Receiving wallet** antes de ir para produção. Faturas de teste não
   precisam dela; uma fatura real sem destino de liquidação falha com
   `409 no_payment_options_available`.

Adicione a chave secreta ao ambiente do seu servidor:

```sh
INVOQ_SECRET_KEY=sk_test_...
```

Se você lida com webhooks, guarde também o segredo do webhook:

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

Comece com as chaves de teste. Troque para a chave de produção e o segredo de webhook de produção
correspondente quando for para produção.

## Crie um cliente

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

Origin de API padrão em produção:

```text
https://api.invoq.money
```

Sobrescreva o origin de API durante o desenvolvimento:

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

`api_origin` precisa ser um origin `http` ou `https` absoluto, sem caminho, query,
hash, usuário ou senha. O SDK anexa os caminhos de recurso `/v1/...`.

O cliente HTTP padrão usa um timeout de requisição de 10 segundos. Use
`InvoqOptions::timeout_ms(...)` para alterá-lo:

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

Passe um `reqwest::Client` personalizado com `InvoqOptions::http_client(...)` quando
precisar de outras configurações de transporte. O `timeout_ms` continua controlando o timeout
do SDK por requisição.

## Faturas

Crie uma fatura:

```rust,no_run
use invoq::{CreateInvoiceInput, Invoq};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;

    let invoice = invoq
        .invoices
        .create(
            CreateInvoiceInput::new("149")
                .description("SaaS boilerplate")
                .reference_id("order_1234")
                .return_url("https://example.com/orders/order_1234"),
        )
        .await?;

    println!("{invoice:?}");
    Ok(())
}
```

Omita `.description(...)`, `.reference_id(...)` ou `.return_url(...)` quando não
forem definidos. Campos opcionais de requisição não definidos são omitidos do JSON. Use
`.without_return_url()` para enviar `return_url: null` e abrir mão da URL de retorno
padrão do projeto.

Use um valor definido no servidor. Não confie em valores vindos do cliente. `amount` é uma
string decimal em USD de `"0.01"` a `"1000000.00"`, com até 2 casas decimais, como
`"129"` ou `"129.99"`. A moeda é sempre USD, e teste ou live vem da chave — nenhum dos dois
é campo da requisição.

Use o `reference_id` para ligar os webhooks `invoice.paid` ao seu pedido. Ele também
deixa a criação segura para repetir: criar de novo com o mesmo `reference_id` e os
mesmos termos da fatura retorna a fatura existente em vez de uma duplicata, enquanto
termos diferentes falham com o erro de API `409 reference_id_conflict`.

Busque uma fatura:

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

`invoices.get()` retorna o formato de fatura pública usado pelo checkout: o formato da
resposta de criação mais `amount_paid`, `project` e `transfers`, sem `reference_id`. Use a
resposta de criação ou o webhook `invoice.paid` quando precisar da sua referência de
comerciante.

Crie um pagamento de teste:

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

`create_test_payment()` só funciona em faturas criadas com uma chave `sk_test_`.
Quando os pagamentos atingem o valor da fatura, ela vira `paid` e a invoq
envia um webhook `invoice.paid` assinado de verdade para a sua URL de webhook de teste. Valores
parciais são permitidos e produzem `partially_paid`.

Omita `.reference_id(...)` quando não for definido; não envie `null` para strings
opcionais de requisição. Strings opcionais não definidas são omitidas do JSON da requisição.

O SDK retorna diretamente o objeto `data` da resposta.

Dois campos de status. `status` é o contábil — `unpaid`, `partially_paid`, `paid`,
`settling`, `settled`, `review_required` — e os três valores equivalentes a pago diferem
apenas em quanto os fundos já andaram até a sua carteira. `checkout_status` é o que o
pagador vê — `open`, `confirming`, `expired`, `paid`, `unavailable` — e nunca autoriza
processar o pedido. `payment_revision` sobe sempre que o conjunto de pagamentos confirmados
muda, então você descarta um snapshot mais antigo do que o que já tem.

Os valores nas respostas são normalizados. Crie com `"129"` e a fatura devolve
`amount: "129.0000"`. Compare valores numericamente, não como texto. `amount_due` é derivado
como `max(amount - amount_paid, 0)` e usa a mesma escala de 18 casas decimais de
`amount_paid`; `amount_overpaid` é o espelho dele, `max(amount_paid - amount, 0)`, então
você nunca precisa subtrair dinheiro por conta própria.

`payment_options` guarda as instruções de pagamento, fixadas na criação e vazio no modo de
teste. Separe primeiro pelo `status` de cada entrada e depois pelo método de cobrança: só
`PaymentOptionStatus::Ready` é pagável, `PaymentInstructions::EvmDeposit` traz
`deposit_address` e `suggested_amount`, e `PaymentInstructions::DirectExact` traz
`recipient_address` e um `exact_amount` que o comprador precisa enviar até o último dígito.
Identifique uma entrada por `chain_namespace`, `chain_reference` e `token_address`, nunca
pela posição dela. `transfers` é o registro confirmado de recebimentos — `transaction_id`,
`event_index`, `amount`, `explorer_transaction_url` — e fica vazio até um pagamento
confirmar. Referência completa:
[documentação da API REST](https://github.com/invoqmoney/api).

Todo enum do wire tem um braço `Unknown`. O backend pode adicionar um valor — uma chain nova, um estado novo — sem tratar isso como quebra, e sem esse braço a resposta inteira falharia na desserialização. O seu `match` ainda precisa lidar com ele, e um status de opção desconhecido nunca é pagável.

## Página de checkout hospedada

Toda fatura também tem uma página de checkout hospedada em:

```text
https://pay.invoq.money/<invoice id>
```

Compartilhe o link ou redirecione para ele quando a janela de checkout dentro da página não encaixar.

## Webhooks

Passe o corpo bruto da requisição para `verify_webhook`. Não interprete o JSON nem o serialize
novamente antes da verificação.

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

`verify_webhook` aceita um `http::HeaderMap`, uma string com o valor do cabeçalho
`invoq-signature` ou tipos de map suportados cujas chaves de cabeçalho não diferenciam
maiúsculas de minúsculas.

Falhas na verificação de webhook retornam `InvoqSignatureVerificationError`.

Use os webhooks `invoice.paid` para processar pedidos no seu servidor.
`invoice_paid_event(&event)` retorna dados tipados para eventos de pagamento que permitem
processamento; `is_invoice_paid(&event)` retorna true para os mesmos eventos quando você só precisa de um
booleano. Use o `reference_id` da fatura para achar e processar o pedido. Os
helpers aceitam status de fatura equivalentes a pagamento confirmado (`paid`, `settling` ou
`settled`) e rejeitam `review_required`. Uma fatura `review_required` ainda não
emite um webhook `invoice.paid`.

A invoq também envia `invoice.payment_reversed` quando uma fatura já paga volta a ficar
abaixo do valor dela — por exemplo, quando uma reorganização da chain derruba uma
transferência confirmada. Capture com `is_invoice_payment_reversed(&event)` ou decodifique
com `invoice_payment_reversed_event(&event)` e segure ou reverta o processamento conforme a
sua própria política. Diferente do helper de pagamento, esse não aplica nenhuma regra de
status: descartar uma reversão deixaria um pedido processado em cima de um pagamento que não
existe mais. Um tipo de evento que esta versão do SDK ainda não modela continua sendo
verificado e devolvido como veio.

Entregas que falham são reenviadas — até 5 tentativas, com intervalos de 1 minuto, 5
minutos, 30 minutos e depois 2 horas — então processe de forma idempotente por
`reference_id` ou pelo `id` da fatura e trate entregas repetidas como uma operação sem
efeito. Elas também podem chegar fora de ordem: fique com o snapshot de maior
`payment_revision`. Responda com 2xx rápido; qualquer outro status conta como entrega
falhada e é reenviado, inclusive redirecionamentos e `4xx`.

O SDK permite uma tolerância de 5 minutos no timestamp. Entregas que falham são assinadas de novo
a cada nova tentativa, então entregas reenviadas comuns ainda passam na verificação dentro dessa
janela. O cabeçalho de assinatura é:

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## Erros

```rust,no_run
use invoq::{CreateInvoiceInput, Invoq, InvoqError};

async fn handle_error(invoq: Invoq) -> Result<(), Box<dyn std::error::Error>> {
    match invoq.invoices.create(CreateInvoiceInput::new("10000000")).await {
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

Validação de entrada da API, falhas de conexão, timeouts e falhas ao interpretar a resposta
retornam `InvoqError`.

## Licença

Licenciado sob a licença MIT.
