# 第16章: Monad Transformer

## はじめに

第8章では `Option` や `Result` を Monad 的に扱う方法を学びました。しかし実際のコードでは、**複数の効果を同時に扱いたい**場面が頻繁に起きます。

例えば「失敗するかもしれない処理 + ログを記録したい」という場合、`Result` だけでも `Option` だけでも不十分です。

**Monad Transformer** はこの「複数の Monad を組み合わせる」問題を解決する技法です。

---

## 問題: ネストした型の扱いにくさ

```rust
fn find_user(id: u32) -> Option<String> { /* ... */ }
fn parse_age(s: &str) -> Result<u32, String> { /* ... */ }

// Option と Result を組み合わせると...
fn process(id: u32) -> Option<Result<u32, String>> {
    let name = find_user(id)?;
    Some(parse_age(&name))
}
```

この型 `Option<Result<u32, String>>` は扱いにくい：

```rust
match process(1) {
    None => println!("ユーザーが見つからない"),
    Some(Ok(age)) => println!("年齢: {}", age),
    Some(Err(e)) => println!("パースエラー: {}", e),
}
```

`and_then` を連鎖させようとすると型が合わなくなります。

---

## OptionResult: 2つの効果を合成する

Rust では Haskell のような汎用 Monad Transformer は型システムの制約で難しいです。代わりに、よく使う組み合わせを専用の型として定義するアプローチが実用的です。

### パターン1: `Result<Option<T>, E>` — 失敗 + 欠損

「処理自体は失敗するかもしれない（`Result`）、かつ値が存在しないこともある（`Option`）」という場合：

```rust
fn find_config(key: &str) -> Result<Option<String>, std::io::Error> {
    // ファイル読み込みに失敗 → Err
    // キーが存在しない → Ok(None)
    // キーが存在する → Ok(Some(value))
    Ok(std::env::var(key).ok())
}

fn main() {
    match find_config("DATABASE_URL") {
        Err(e) => eprintln!("設定ファイル読み込みエラー: {}", e),
        Ok(None) => println!("DATABASE_URL は未設定"),
        Ok(Some(url)) => println!("DB: {}", url),
    }
}
```

### パターン2: `Option<Result<T, E>>` — 欠損 + 失敗

「そもそも値が存在しないこともある（`Option`）、存在するなら変換に失敗するかもしれない（`Result`）」という場合：

```rust
fn parse_optional_number(s: Option<&str>) -> Option<Result<i32, std::num::ParseIntError>> {
    s.map(|v| v.parse::<i32>())
}
```

これを `transpose()` で `Result<Option<T>, E>` に変換できます：

```rust
fn parse_optional_number(s: Option<&str>) -> Result<Option<i32>, std::num::ParseIntError> {
    s.map(|v| v.parse::<i32>()).transpose()
}

fn main() {
    println!("{:?}", parse_optional_number(None));        // Ok(None)
    println!("{:?}", parse_optional_number(Some("42")));  // Ok(Some(42))
    println!("{:?}", parse_optional_number(Some("abc"))); // Err(ParseIntError)
}
```

`transpose()` は `Option<Result<T, E>>` ↔ `Result<Option<T>, E>` を変換する標準ライブラリのメソッドです。

---

## Writer パターン: 計算結果 + ログ

「値を計算しながらログを蓄積したい」という場合、Writer Monad に相当するパターンを使います。

```rust
/// 計算結果と付随するログをペアで持つ型
#[derive(Debug, Clone)]
struct Writer<A> {
    value: A,
    log: Vec<String>,
}

impl<A> Writer<A> {
    fn new(value: A) -> Self {
        Writer { value, log: Vec::new() }
    }

    fn tell(mut self, message: impl Into<String>) -> Self {
        self.log.push(message.into());
        self
    }

    fn map<B, F>(self, f: F) -> Writer<B>
    where
        F: FnOnce(A) -> B,
    {
        Writer {
            value: f(self.value),
            log: self.log,
        }
    }

    fn and_then<B, F>(self, f: F) -> Writer<B>
    where
        F: FnOnce(A) -> Writer<B>,
    {
        let mut result = f(self.value);
        let mut combined_log = self.log;
        combined_log.append(&mut result.log);
        Writer {
            value: result.value,
            log: combined_log,
        }
    }
}

fn double_with_log(n: i32) -> Writer<i32> {
    Writer::new(n * 2).tell(format!("{} を2倍にした → {}", n, n * 2))
}

fn add_ten_with_log(n: i32) -> Writer<i32> {
    Writer::new(n + 10).tell(format!("{} に10を足した → {}", n, n + 10))
}

fn main() {
    let result = Writer::new(5)
        .and_then(double_with_log)
        .and_then(add_ten_with_log);

    println!("最終値: {}", result.value); // 20
    for entry in &result.log {
        println!("  ログ: {}", entry);
    }
    // ログ: 5 を2倍にした → 10
    // ログ: 10 に10を足した → 20
}
```

---

## State パターン: 計算結果 + 状態

「計算しながら状態を引き回したい」場合、State Monad に相当するパターンを使います。

```rust
/// 状態 S を受け取り、(結果 A, 新しい状態 S) を返す関数をラップした型
struct State<S, A> {
    run: Box<dyn FnOnce(S) -> (A, S)>,
}

impl<S: 'static, A: 'static> State<S, A> {
    fn new<F>(f: F) -> Self
    where
        F: FnOnce(S) -> (A, S) + 'static,
    {
        State { run: Box::new(f) }
    }

    fn run_state(self, s: S) -> (A, S) {
        (self.run)(s)
    }

    fn map<B: 'static, F>(self, f: F) -> State<S, B>
    where
        F: FnOnce(A) -> B + 'static,
    {
        State::new(move |s| {
            let (a, s2) = self.run_state(s);
            (f(a), s2)
        })
    }

    fn and_then<B: 'static, F>(self, f: F) -> State<S, B>
    where
        F: FnOnce(A) -> State<S, B> + 'static,
    {
        State::new(move |s| {
            let (a, s2) = self.run_state(s);
            f(a).run_state(s2)
        })
    }
}

/// 現在の状態を取得する
fn get<S: Clone + 'static>() -> State<S, S> {
    State::new(|s: S| (s.clone(), s))
}

/// 状態を更新する
fn put<S: 'static>(new_state: S) -> State<S, ()> {
    State::new(|_| ((), new_state))
}

fn main() {
    // カウンターをインクリメントしながら値を記録する例
    let computation = get::<i32>()
        .and_then(|n| {
            put(n + 1).and_then(move |_| {
                get::<i32>().map(move |m| (n, m))
            })
        });

    let (result, final_state) = computation.run_state(10);
    println!("初期値: {}, 最終値: {}", result.0, result.1); // 初期値: 10, 最終値: 11
    println!("最終状態: {}", final_state); // 11
}
```

---

## 実用的なアプローチ: エラー型の統合

Rust で最も実用的な「Monad Transformer 的」テクニックは、`?` 演算子と `From` トレイトによる**エラー型の統合**です。

```rust
use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
enum AppError {
    Parse(String),
    Logic(String),
    NotFound,
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e.to_string())
    }
}

fn parse_positive(s: &str) -> Result<u32, AppError> {
    let n: i32 = s.parse()?; // ParseIntError → AppError::Parse に自動変換
    if n < 0 {
        return Err(AppError::Logic(format!("{} は負の数", n)));
    }
    Ok(n as u32)
}

fn lookup_and_parse(data: &[(&str, &str)], key: &str) -> Result<u32, AppError> {
    let value = data.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
        .ok_or(AppError::NotFound)?; // Option → Result に変換

    parse_positive(value) // Result<u32, AppError>
}

fn main() {
    let data = vec![("age", "25"), ("score", "-5"), ("name", "Alice")];

    println!("{:?}", lookup_and_parse(&data, "age"));   // Ok(25)
    println!("{:?}", lookup_and_parse(&data, "score")); // Err(Logic(...))
    println!("{:?}", lookup_and_parse(&data, "missing")); // Err(NotFound)
    println!("{:?}", lookup_and_parse(&data, "name"));  // Err(Parse(...))
}
```

`?` 演算子が自動的に `From` 変換を呼び出すため、複数のエラー型を一つに統合しながら `and_then` を連鎖させたのと同等の効果が得られます。

---

## まとめ

| パターン | Haskell での対応 | Rust での実現方法 |
|---------|----------------|-----------------|
| 失敗 + 欠損 | `MaybeT Either` | `Result<Option<T>, E>` + `transpose()` |
| 計算 + ログ | `WriterT` | `Writer<A>` 構造体 + `and_then` |
| 計算 + 状態 | `StateT` | `State<S, A>` 構造体 + `and_then` |
| 複数エラー統合 | `ExceptT` | `From` トレイト + `?` 演算子 |

Rust では Haskell ほど汎用的な Monad Transformer は作りにくいですが、**よく使う組み合わせを専用の型や標準ライブラリのメソッドで解決する**アプローチが実用的です。

---

## よくある落とし穴と対処法

**落とし穴1: `Option<Result<T, E>>` と `Result<Option<T>, E>` を混同する**

どちらが正しいかは「何が主な効果か」で決まります。「処理は必ず実行されるが値がないこともある」なら `Result<Option<T>, E>`、「そもそも処理するものがないこともある」なら `Option<Result<T, E>>` が自然です。

**落とし穴2: `anyhow` や `thiserror` を使わずに手書きエラー変換をしすぎる**

実用プロジェクトでは `thiserror` クレートで `From` 実装を自動生成し、`anyhow` で動的エラー型を使うことで Monad Transformer 的な効果が簡単に得られます。

---

## 章末演習問題

1. `Vec<Option<i32>>` に対して、`None` を `0` として扱い、全要素の合計を計算する関数を `transpose` と `flatten` を使わずに書いてください。次に `transpose` を使って書き直してください。

2. `Writer<i32>` を使って「フィボナッチ数を計算しながら各ステップをログする」関数を実装してください。

3. `State<Vec<i32>, ()>` を使って、スタックに push/pop する操作を実装してください。
