# 第2章: Rustにおける不変性と所有権

## はじめに

関数型プログラミングの中心的な概念のひとつが **不変性（Immutability）** です。値を変更するのではなく、変換した新しい値を作り出すという考え方です。Rustはシステムプログラミング言語でありながら、この思想を言語設計の根幹に組み込んでいます。本章では、Rustの不変性と所有権が関数型スタイルとどのように対応しているかを学びます。

---

## 1. デフォルト不変性 — `let` と `let mut`

Rustでは、変数はデフォルトで **不変（immutable）** です。変更可能にするには明示的に `let mut` と書く必要があります。これは「変更は例外」という関数型プログラミングの考え方そのものです。

```rust
fn main() {
    let x = 5;
    // x = 6; // コンパイルエラー: cannot assign twice to immutable variable

    let mut y = 5;
    y = 6; // OK: mut を明示しているので変更できる
    println!("y = {y}");
}
```

### 関数型的な意味

命令型プログラミングでは「状態を変化させながら処理を進める」のが自然です。一方、関数型プログラミングでは「値から新しい値を導く」のが基本スタイルです。

```rust
// 命令型スタイル: 変数を書き換えながら計算
fn sum_imperative(values: &[i32]) -> i32 {
    let mut total = 0;
    for v in values {
        total += v; // 状態 total を更新し続ける
    }
    total
}

// 関数型スタイル: fold で値を畳み込む（状態を持たない）
fn sum_functional(values: &[i32]) -> i32 {
    values.iter().fold(0, |acc, &v| acc + v)
}
```

`let mut` を見かけたら「ここに副作用がある」というシグナルだと考えてください。不変バインディングを増やすほどコードは追いやすくなります。

---

## 2. 所有権と「値の変換」

Rustの所有権システムは、値が「どこか1か所にしか存在しない」ことを保証します。これは関数型プログラミングの **値渡し（pass by value）** と自然に対応します。

関数が所有権を受け取って新しい値を返すパターンは、まさに「変換」です。元の値は消費され、新しい値が生まれます。

```rust
/// 文字列を受け取り、先頭を大文字にした新しい String を返す。
/// 元の String の所有権は関数内で消費される。
fn capitalize(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}

fn main() {
    let original = String::from("hello");
    let capitalized = capitalize(original);
    // original はここでは使えない（所有権が移動したため）
    println!("{capitalized}"); // "Hello"
}
```

この「消費して新しい値を返す」パターンは、元の値を変更しないという点で純粋関数的です。

### 値の変換チェーン

所有権の移動はチェーンとして連結できます。各ステップが入力を変換して次の出力を作ります。

```rust
fn add_exclamation(s: String) -> String {
    s + "!"
}

fn to_uppercase_owned(s: String) -> String {
    s.to_uppercase()
}

fn main() {
    let result = add_exclamation(to_uppercase_owned(String::from("hello")));
    println!("{result}"); // "HELLO!"
}
```

---

## 3. 借用と純粋関数

すべての関数が所有権を受け取る必要はありません。値を「読むだけ」なら **共有参照（`&T`）** で借用します。借用して計算し、副作用なく結果を返す関数は **純粋関数** と同じ性質を持ちます。

```rust
/// スライスの合計を返す純粋関数。
/// 入力を変更せず、外部状態にも依存しない。
fn sum(values: &[i32]) -> i32 {
    values.iter().sum()
}

/// 文字列が回文かどうかを判定する純粋関数。
fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let reversed: Vec<char> = chars.iter().rev().cloned().collect();
    chars == reversed
}
```

`&T`（共有参照）を取る関数は、コンパイラにより「値を変更しない」ことが保証されます。これは純粋関数の条件のひとつである **副作用がないこと** をコンパイル時に強制するものです。

### `&mut T` との違い

`&mut T`（可変参照）を取る関数は値を書き換えられます。これは副作用を持つ関数であり、純粋関数とは区別されます。

```rust
/// 純粋関数: 参照を受け取り、新しいVecを返す。元は変更しない。
fn doubled(values: &[i32]) -> Vec<i32> {
    values.iter().map(|&v| v * 2).collect()
}

/// 副作用を持つ関数: インプレースで変更する。
fn double_in_place(values: &mut Vec<i32>) {
    for v in values.iter_mut() {
        *v *= 2;
    }
}
```

関数型スタイルでは前者（`&T` を取り新しい値を返す）を好みます。

---

## 4. `const` と `static` の使いどころ

### `const` — コンパイル時定数

`const` はコンパイル時に評価される定数です。型注釈が必須で、スコープのどこにでも置けます。関数型プログラミングにおける「名前付き定数」に相当します。

```rust
const MAX_RETRIES: u32 = 3;
const PI: f64 = std::f64::consts::PI;
const GREETING: &str = "Hello, functional world!";

fn circle_area(radius: f64) -> f64 {
    PI * radius * radius
}
```

`const` は **インライン展開** されるため、ゼロコスト抽象です。

### `static` — プログラム全体で唯一のメモリ上の場所

`static` はプログラムの起動から終了まで存在するグローバル変数です。`const` と異なり、一定のメモリアドレスを持ちます。文字列リテラルや共有設定などに使います。

```rust
static APP_NAME: &str = "FP Study";
static PRIMES: [u32; 5] = [2, 3, 5, 7, 11];
```

`static mut` は存在しますが、安全でない（`unsafe`）ため原則として避けてください。可変グローバル状態は副作用の温床であり、関数型スタイルと相性が悪いです。

### 使い分けの指針

| | `const` | `static` |
|---|---|---|
| 評価タイミング | コンパイル時 | コンパイル時 |
| メモリアドレス | 不定（インライン展開） | 固定 |
| 参照を取れるか | 可（`&const` として） | 可（`'static` ライフタイム） |
| 主な用途 | 数値定数・短い文字列 | 共有文字列・固定配列 |

---

## 5. 不変データ構造のパターン

### struct update syntax で「更新された値」を作る

Rustの `struct` はデフォルトで不変です。フィールドを「変更」したいとき、元の値は変えずに **新しいインスタンスを作る** のが関数型スタイルです。`..` (struct update syntax) を使うと、変えたいフィールドだけ指定して残りを既存インスタンスからコピーできます。

```rust
#[derive(Debug, Clone, PartialEq)]
struct Config {
    host: String,
    port: u16,
    max_connections: u32,
    timeout_secs: u64,
}

impl Config {
    fn new() -> Self {
        Config {
            host: String::from("localhost"),
            port: 8080,
            max_connections: 100,
            timeout_secs: 30,
        }
    }

    /// ポートだけ変えた新しい Config を返す。元の Config は変更しない。
    fn with_port(&self, port: u16) -> Self {
        Config {
            port,
            ..self.clone() // 残りのフィールドは self からコピー
        }
    }

    /// ホストだけ変えた新しい Config を返す。
    fn with_host(&self, host: impl Into<String>) -> Self {
        Config {
            host: host.into(),
            ..self.clone()
        }
    }
}

fn main() {
    let default_config = Config::new();
    let prod_config = default_config
        .with_host("example.com")
        .with_port(443);

    println!("default: {default_config:?}");
    println!("prod:    {prod_config:?}");
    // default_config は変更されていない
}
```

このパターンは **Builder パターン** や **Lensパターン** と呼ばれ、関数型言語でよく使われます。

### 不変な変換パイプライン

元のコレクションを変えずに変換結果を得るパターンです。

```rust
fn process_scores(scores: &[u32]) -> Vec<u32> {
    scores
        .iter()
        .filter(|&&s| s >= 60)      // 60点以上を抽出
        .map(|&s| s + 5)            // 5点加算（補正）
        .collect()
}

fn main() {
    let scores = vec![45, 72, 88, 55, 91, 60];
    let processed = process_scores(&scores);
    println!("original:  {scores:?}");   // 変更なし
    println!("processed: {processed:?}");
}
```

---

## まとめ

| Rustの機能 | 関数型プログラミングとの対応 |
|---|---|
| `let`（デフォルト不変） | 不変性をデフォルトとする設計 |
| `let mut` の明示 | 副作用の可視化 |
| 所有権の移動 | 値の変換（transform） |
| `&T` による借用 | 純粋関数（入力を変えない） |
| `const` / `static` | 名前付き定数 |
| struct update syntax | 不変データの「更新」（新インスタンス生成） |

不変性を中心に設計することで、コードの予測可能性が高まり、バグが減り、並行処理にも強くなります。

---

## 章末演習問題

### 問題1: 不変バインディングによる変換

以下の命令型コードを、`let mut` を一切使わず `let` のみで書き直してください。イテレータを活用してください。

```rust
fn calculate_imperative(values: &[i32]) -> i32 {
    let mut result = 0;
    for &v in values {
        if v > 0 {
            result += v * 2;
        }
    }
    result
}
```

ヒント: `filter` と `map` と `sum` を組み合わせてみましょう。

---

### 問題2: struct update syntax

以下の `User` 構造体に対して、`email` だけを変えた新しい `User` を返すメソッド `with_email` を実装してください。元の `User` は変更せず、`Clone` を使って新しいインスタンスを返してください。

```rust
#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
    age: u32,
}
```

---

### 問題3: 純粋関数の設計

以下の関数は副作用がありますか？それぞれについて「純粋関数かどうか」と「その理由」を答えてください。

```rust
// (a)
fn double(x: i32) -> i32 {
    x * 2
}

// (b)
fn print_and_return(x: i32) -> i32 {
    println!("{x}");
    x
}

// (c)
fn append_suffix(s: &str) -> String {
    format!("{s}_processed")
}

// (d)
fn increment(counter: &mut i32) {
    *counter += 1;
}
```

---

## よくある落とし穴と対処法

### 落とし穴1: `mut` を付けすぎる

```rust
// NG: 不必要な mut
let mut result = Vec::new();
for x in &data {
    result.push(x * 2);
}

// OK: イテレータで不変に変換
let result: Vec<_> = data.iter().map(|&x| x * 2).collect();
```

### 落とし穴2: 内部可変性（`Cell` / `RefCell`）の乱用

`RefCell<T>` は実行時にチェックするため、パニックのリスクがあります。

```rust
// 危険: 複数の可変借用でパニック
let cell = RefCell::new(vec![1, 2, 3]);
let _borrow1 = cell.borrow_mut();
let _borrow2 = cell.borrow_mut(); // パニック！
```

**対処法:** 内部可変性は本当に必要な場合のみ使い、スコープを最小限に。

### 落とし穴3: `clone()` の多用でパフォーマンス低下

```rust
// NG: 不必要なクローン
fn process(data: Vec<i32>) -> Vec<i32> {
    let cloned = data.clone(); // 不要なコピー
    cloned.iter().map(|&x| x * 2).collect()
}

// OK: 参照を受け取る
fn process(data: &[i32]) -> Vec<i32> {
    data.iter().map(|&x| x * 2).collect()
}
```
