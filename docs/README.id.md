# invoq Rust SDK

[English](https://github.com/invoqmoney/sdk-rust/blob/main/README.md) · **Bahasa Indonesia** · [Español](./README.es-419.md) · [Français](./README.fr.md) · [Português](./README.pt-BR.md) · [Tiếng Việt](./README.vi.md) · [Türkçe](./README.tr.md) · [ไทย](./README.th.md) · [简体中文](./README.zh-Hans.md) · [繁體中文](./README.zh-Hant.md)

> Dokumen ini terjemahan dari README bahasa Inggris; kalau ada perbedaan, [versi bahasa Inggris](https://github.com/invoqmoney/sdk-rust/blob/main/README.md) yang berlaku.

SDK Rust untuk API server invoq dan verifikasi webhook.

Gunakan crate ini hanya di server Anda. Crate ini menerima kunci rahasia dan tidak
boleh ikut dikompilasi ke dalam aplikasi sisi klien.

**Coding pakai AI? Tempelkan ini.**

```text
Tambahkan pembayaran stablecoin ke proyek saya dengan invoq. Mulai dari mode tes. Baca dokumentasinya sebelum menulis kode: https://invoq.money/llms.txt
```

## SDK server

Buat invoice dan verifikasi webhook dari backend Anda dalam bahasa mana pun berikut — REST API dan tanda tangan webhook-nya sama persis. Repo ini adalah SDK Rust.

| Bahasa | Repositori |
| --- | --- |
| Node.js | [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js) (`@invoq/server`) |
| Python | [github.com/invoqmoney/sdk-python](https://github.com/invoqmoney/sdk-python) |
| PHP | [github.com/invoqmoney/sdk-php](https://github.com/invoqmoney/sdk-php) |
| Go | [github.com/invoqmoney/sdk-go](https://github.com/invoqmoney/sdk-go) |
| Rust | **repo ini** |
| Ruby | [github.com/invoqmoney/sdk-ruby](https://github.com/invoqmoney/sdk-ruby) |

Sisi browser-nya sama untuk setiap backend: **`@invoq/checkout`** (JavaScript, di [github.com/invoqmoney/sdk-js](https://github.com/invoqmoney/sdk-js)) membuka jendela checkout yang tertanam di halaman untuk frontend apa pun.

## Instalasi

```sh
cargo add invoq
cargo add tokio --features macros,rt-multi-thread
```

SDK ini memakai `reqwest` dan Rust async.

Membutuhkan Rust 1.86 atau lebih baru.

## Siapkan kunci Anda

1. Masuk ke [dashboard invoq](https://app.invoq.money) dan buat sebuah
   proyek.
2. Di halaman **API keys**, buat kunci rahasia (secret key). Kunci uji coba diawali `sk_test_`,
   kunci produksi diawali `sk_live_`. Mode kuncinya menentukan apakah invoice yang dibuat itu uji coba
   atau produksi.
3. Di pengaturan **webhooks** proyek Anda, simpan URL webhook Anda. Kunci rahasia webhook
   (`whsec_...`) untuk mode itu hanya ditampilkan sekali, saat webhook pertama kali diaktifkan —
   jadi langsung simpan. URL webhook harus berupa URL HTTPS yang bisa diakses publik.
4. Siapkan **Receiving wallet** Anda sebelum go live. Invoice uji coba tidak membutuhkannya;
   invoice live tanpa tujuan penyelesaian gagal dengan `409 no_payment_options_available`.

Tambahkan kunci rahasia ke lingkungan server Anda:

```sh
INVOQ_SECRET_KEY=sk_test_...
```

Kalau Anda menangani webhook, simpan juga kunci rahasia webhook-nya:

```sh
INVOQ_WEBHOOK_SECRET=whsec_...
```

Mulailah dengan kunci uji coba. Ganti ke kunci produksi dan kunci rahasia webhook produksi
yang sesuai saat masuk produksi.

## Buat klien

```rust,no_run
use invoq::Invoq;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _invoq = Invoq::new(std::env::var("INVOQ_SECRET_KEY")?)?;
    Ok(())
}
```

Origin API bawaan di produksi:

```text
https://api.invoq.money
```

Timpa origin API saat pengembangan:

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

`api_origin` harus berupa origin `http` atau `https` absolut tanpa path, query,
hash, username, atau password. SDK menambahkan path sumber daya `/v1/...`.

Klien HTTP bawaan memakai timeout request 10 detik. Pakai
`InvoqOptions::timeout_ms(...)` untuk mengubahnya:

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

Berikan `reqwest::Client` kustom lewat `InvoqOptions::http_client(...)` kalau Anda
butuh pengaturan transport yang berbeda. `timeout_ms` tetap mengontrol timeout SDK
per request.

## Invoice

Buat invoice:

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

Lewati `.description(...)`, `.reference_id(...)`, atau `.return_url(...)` kalau tidak
diisi. Field request opsional yang tidak diisi akan dihilangkan dari JSON. Pakai
`.without_return_url()` untuk mengirim `return_url: null` dan menolak return URL bawaan
proyek.

Tentukan jumlahnya di sisi server. Jangan percaya jumlah yang dikirim klien. `amount` adalah
string desimal USD dari `"0.01"` sampai `"1000000.00"` dengan maksimal 2 angka di belakang koma,
misalnya `"129"` atau `"129.99"`. Mata uangnya selalu USD, dan mode uji coba atau live
ditentukan oleh kuncinya — keduanya bukan field permintaan.

Pakai `reference_id` untuk memetakan webhook `invoice.paid` kembali ke pesanan Anda. Ini juga
membuat pembuatan invoice aman diulang: membuat lagi dengan `reference_id` yang sama dan
ketentuan invoice yang sama akan mengembalikan invoice yang sudah ada alih-alih duplikat,
sementara ketentuan yang berbeda gagal dengan error API `409 reference_id_conflict`.

Ambil invoice:

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

`invoices.get()` mengembalikan bentuk invoice publik yang dipakai checkout: bentuk respons
pembuatan ditambah `amount_paid`, `project`, dan `transfers`, dikurangi `reference_id`. Gunakan
respons pembuatan atau webhook `invoice.paid` saat Anda butuh referensi merchant Anda.

Buat pembayaran uji coba:

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

`create_test_payment()` hanya bekerja pada invoice yang dibuat dengan kunci `sk_test_`.
Begitu pembayaran mencapai jumlah invoice, invoice menjadi `paid` dan invoq mengirim
webhook `invoice.paid` bertanda tangan sungguhan ke URL webhook uji coba Anda. Jumlah
parsial diperbolehkan dan menghasilkan `partially_paid`.

Lewati `.reference_id(...)` kalau tidak diisi; jangan mengirim `null` untuk string request
opsional. String opsional yang tidak diisi dihilangkan dari JSON request.

SDK mengembalikan objek `data` dari respons secara langsung.

Dua field status. `status` adalah status pembukuan — `unpaid`, `partially_paid`, `paid`,
`settling`, `settled`, `review_required` — dan tiga nilai yang berarti sudah dibayar hanya
berbeda pada seberapa jauh dananya bergerak ke dompet Anda. `checkout_status` adalah status
yang dilihat pembayar — `open`, `confirming`, `expired`, `paid`, `unavailable` — dan tidak
pernah menjadi izin memproses pesanan. `payment_revision` naik setiap kali kumpulan pembayaran
terkonfirmasi berubah, jadi Anda bisa membuang snapshot yang lebih lama dari yang sudah Anda
pegang.

Jumlah di respons dinormalkan. Buat dengan `"129"` dan invoice mengembalikan
`amount: "129.0000"`. Bandingkan jumlah secara numerik, bukan sebagai string. `amount_due`
diturunkan sebagai `max(amount - amount_paid, 0)` dan memakai skala 18 desimal yang sama dengan
`amount_paid`; `amount_overpaid` adalah kebalikannya, `max(amount_paid - amount, 0)`, jadi Anda
tidak perlu mengurangkannya sendiri.

`payment_options` berisi instruksi pembayarannya, ditetapkan saat pembuatan dan kosong di mode
uji coba. Bedakan dulu lewat `status` tiap entri, lalu lewat metode penagihannya: hanya
`PaymentOptionStatus::Ready` yang bisa dibayar, `PaymentInstructions::EvmDeposit` membawa
`deposit_address` dan `suggested_amount`, dan `PaymentInstructions::DirectExact` membawa
`recipient_address` serta `exact_amount` yang harus dikirim pembeli persis sampai digit
terakhir. Kenali sebuah entri lewat `chain_namespace`, `chain_reference`, dan `token_address`,
jangan lewat posisinya. `transfers` adalah jejak penerimaan terkonfirmasi — `transaction_id`,
`event_index`, `amount`, `explorer_transaction_url` — dan tetap kosong sampai ada pembayaran
yang terkonfirmasi. Referensi field lengkap:
[dokumen REST API](https://github.com/invoqmoney/api).

Setiap enum di wire punya arm `Unknown`. Backend bisa menambah nilai — chain baru, state baru — tanpa menganggapnya breaking, dan tanpa arm itu seluruh respons akan gagal dideserialisasi. `match` Anda tetap harus menanganinya, dan status opsi yang tidak dikenal tidak pernah bisa dibayar.

## Halaman checkout yang dihosting

Setiap invoice juga punya halaman checkout yang dihosting di:

```text
https://pay.invoq.money/<invoice id>
```

Bagikan tautannya atau alihkan ke sana kalau jendela checkout dalam halaman kurang pas.

## Webhook

Berikan isi request mentah ke `verify_webhook`. Jangan mem-parse JSON lalu
men-serialisasinya ulang sebelum verifikasi.

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

`verify_webhook` menerima `http::HeaderMap`, string yang berisi nilai header
`invoq-signature`, atau tipe map yang didukung dengan kunci header yang tidak peka
huruf besar-kecil.

Kegagalan verifikasi webhook mengembalikan `InvoqSignatureVerificationError`.

Gunakan webhook `invoice.paid` untuk memproses pesanan di server Anda.
`invoice_paid_event(&event)` mengembalikan data bertipe untuk event terbayar yang bisa
diproses; `is_invoice_paid(&event)` mengembalikan true untuk event yang sama kalau Anda
hanya butuh nilai boolean. Pakai `reference_id` invoice untuk menemukan dan memproses
pesanan Anda. Kedua helper ini menerima status invoice yang setara dengan sudah dibayar
(`paid`, `settling`, atau `settled`) dan menolak `review_required`. Invoice dengan status
`review_required` belum mengirimkan webhook `invoice.paid`.

invoq juga mengirim `invoice.payment_reversed` ketika invoice yang tadinya lunas turun lagi di
bawah jumlahnya — misalnya karena reorg rantai membatalkan transfer yang sudah terkonfirmasi.
Tangkap event itu dengan `is_invoice_payment_reversed(&event)` atau baca datanya dengan
`invoice_payment_reversed_event(&event)`, lalu tahan atau batalkan pemrosesan sesuai kebijakan
Anda sendiri. Berbeda dengan helper pembayaran, helper ini tidak menerapkan aturan status apa
pun: membuang satu pembatalan akan meninggalkan pesanan yang sudah diproses atas pembayaran
yang tidak ada lagi. Tipe event yang belum dikenal versi SDK ini tetap lolos verifikasi dan
dikembalikan apa adanya.

Pengiriman yang gagal akan diulang — sampai 5 kali, dengan jeda 1 menit, 5 menit, 30 menit,
lalu 2 jam — jadi proses pesanan secara idempoten berdasarkan `reference_id` atau `id` invoice
dan jadikan pengiriman berulang sebagai operasi tanpa efek. Urutan kedatangannya juga tidak
dijamin: simpan snapshot dengan `payment_revision` tertinggi. Balas 2xx secepatnya; status lain
apa pun dihitung sebagai pengiriman gagal dan akan diulang, termasuk redirect dan `4xx`.

SDK mengizinkan toleransi timestamp 5 menit. Pengiriman yang gagal ditandatangani ulang
pada tiap percobaan ulang, jadi pengiriman ulang yang normal tetap lolos verifikasi dalam
rentang waktu itu. Header tanda tangannya adalah:

```text
invoq-signature: t=<unix seconds>,v1=<hex HMAC-SHA256 of "<t>.<raw body>">
```

## Error

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

Validasi input API, kegagalan koneksi, timeout, dan kegagalan mem-parse respons
mengembalikan `InvoqError`.

## Lisensi

Dilisensikan di bawah lisensi MIT.
