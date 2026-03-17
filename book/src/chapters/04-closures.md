# クロージャ（Closures）

## 概要

**クロージャ**とは、定義されたスコープの変数をキャプチャできる匿名関数です。Rust では `|引数| 式` の形で記述します。

## 基本的な構文

```rust
// 通常の関数
fn add(a: i32, b: i32) -> i32 { a + b }

// クロージャ（引数の型を省略できる）
let add = |a, b| a + b;
let square = |x: i32| x * x;
let greet = |name| format!("こんにちは、{}!", name);
```

## 環境のキャプチャ

クロージャは定義されたスコープの変数を「キャプチャ」できます：

```rust
let base = 100;

// base をキャプチャ（参照でキャプチャ）
let add_base = |x: i32| x + base;
println!("{}", add_base(42)); // 142

// move でキャプチャ（所有権ごと移動）
let greeting = String::from("Hello");
let say_hello = move || println!("{}", greeting);
say_hello();
```

## Fn、FnMut、FnOnce

Rust のクロージャには3種類のトレイトがあります：

| トレイト | 説明 |
|---------|------|
| `Fn`    | 不変参照でキャプチャ。何度でも呼び出せる |
| `FnMut` | 可変参照でキャプチャ。何度でも呼び出せる |
| `FnOnce`| 所有権をキャプチャ。1度だけ呼び出せる |

```rust
// FnMut の例: カウンター
fn make_counter(start: i32) -> impl FnMut() -> i32 {
    let mut count = start;
    move || {
        let current = count;
        count += 1;
        current
    }
}

let mut counter = make_counter(0);
println!("{}", counter()); // 0
println!("{}", counter()); // 1
println!("{}", counter()); // 2
```

## 実践例: アキュムレータ

```rust
fn make_accumulator(initial: f64) -> impl FnMut(f64) -> f64 {
    let mut total = initial;
    move |n| {
        total += n;
        total
    }
}

let mut acc = make_accumulator(0.0);
println!("{}", acc(10.0)); // 10.0
println!("{}", acc(20.0)); // 30.0
println!("{}", acc(5.0));  // 35.0
```

## イテレータとの組み合わせ

クロージャはイテレータと組み合わせて使うことが多いです：

```rust
let numbers = vec![1, 2, 3, 4, 5];
let threshold = 3;

// クロージャで環境をキャプチャしながらフィルタリング
let big: Vec<i32> = numbers
    .iter()
    .filter(|&&n| n > threshold)
    .cloned()
    .collect();
// [4, 5]
```

## 演習

`crates/closures/src/main.rs` を参照してください。

```bash
cargo run -p closures
cargo test -p closures
```
