# 第6章: Option型とResult型による関数型エラーハンドリング

## 6.1 エラーハンドリングの関数型アプローチ

命令型言語では「例外（exception）」や「null 参照」がエラーや欠損値を表すために広く使われています。しかしこれらには重大な問題があります。

- **null 参照**: コンパイラが「値が存在しない可能性」を追跡できないため、実行時に `NullPointerException` が発生する
- **例外**: 制御フローが関数のシグネチャから不可視になり、どこで例外が飛ぶか静的に把握しづらい

Rust はこれらの問題を**型システムで解決**します。

| 問題 | Rust の解決策 |
|------|--------------|
| 値が存在しないかもしれない | `Option<T>` |
| 処理が失敗するかもしれない | `Result<T, E>` |

どちらも**代数的データ型（Algebraic Data Type）**であり、関数型プログラミングの中核概念です。コンパイラが「失敗の可能性」を追跡するため、処理し忘れがあればコンパイルエラーになります。

---

## 6.2 `Option<T>` — null の代替

`Option<T>` は「値がある（`Some(T)`）」か「値がない（`None`）」かを型で表します。

```rust
enum Option<T> {
    Some(T),  // 値が存在する
    None,     // 値が存在しない
}
```

### 基本的な使い方

```rust
/// 文字列スライスを整数にパースする。変換できなければ None を返す。
fn parse_positive(s: &str) -> Option<u32> {
    s.parse::<u32>().ok() // Result を Option に変換
}

fn main() {
    let a = parse_positive("42");  // Some(42)
    let b = parse_positive("-1");  // None（u32 は負数を表せない）
    let c = parse_positive("abc"); // None（数値でない）

    // パターンマッチで値を取り出す
    match a {
        Some(n) => println!("パース成功: {}", n),
        None    => println!("パース失敗"),
    }

    // if let で簡潔に書く
    if let Some(n) = b {
        println!("値: {}", n);
    } else {
        println!("値なし");
    }
}
```

### `Option` を返す標準ライブラリの関数

```rust
fn option_examples() {
    let v = vec![1, 2, 3];

    // スライスの最初の要素（空の場合は None）
    let first: Option<&i32> = v.first();

    // HashMap の検索（キーが存在しない場合は None）
    let mut map = std::collections::HashMap::new();
    map.insert("key", 100u32);
    let val: Option<&u32> = map.get("key");
    let missing: Option<&u32> = map.get("no_such_key");

    println!("first={:?}, val={:?}, missing={:?}", first, val, missing);
}
```

---

## 6.3 `Result<T, E>` — 例外の代替

`Result<T, E>` は「成功（`Ok(T)`）」か「失敗（`Err(E)`）」かを型で表します。

```rust
enum Result<T, E> {
    Ok(T),   // 成功した値
    Err(E),  // エラーを表す値
}
```

### 基本的な使い方

```rust
use std::num::ParseIntError;

/// 文字列を整数にパースする。失敗したときのエラーを型で表す。
fn parse_int(s: &str) -> Result<i32, ParseIntError> {
    s.parse::<i32>() // 標準ライブラリが Result を返す
}

fn main() {
    let ok = parse_int("42");   // Ok(42)
    let err = parse_int("abc"); // Err(ParseIntError { ... })

    match ok {
        Ok(n)  => println!("成功: {}", n),
        Err(e) => println!("失敗: {}", e),
    }

    // is_ok() / is_err() で判定だけ行う
    println!("ok? {}", err.is_ok());  // false
    println!("err? {}", err.is_err()); // true
}
```

---

## 6.4 `Option` の関数型チェーン

`Option` には関数型スタイルで値を変換・連鎖するメソッドが豊富に揃っています。

### `map` — 値を変換する

`Some(x)` なら `Some(f(x))`、`None` ならそのまま `None` を返します。

```rust
fn map_example() {
    let maybe_str: Option<&str> = Some("42");

    // Some("42") → Some(42) と変換
    let maybe_num: Option<i32> = maybe_str.map(|s| s.parse::<i32>().unwrap_or(0));

    println!("{:?}", maybe_num); // Some(42)

    let nothing: Option<&str> = None;
    let still_nothing = nothing.map(|s| s.len());
    println!("{:?}", still_nothing); // None（map はクロージャを呼ばない）
}
```

### `and_then` — Option を返す処理を連鎖する（flatMap）

`map` が `Option<Option<T>>` になるケースで使います。Haskell の `>>=`（bind）に相当します。

```rust
/// 文字列を u32 にパースする
fn parse_u32(s: &str) -> Option<u32> {
    s.parse::<u32>().ok()
}

/// 値が 100 以下なら Some、そうでなければ None
fn check_max(n: u32) -> Option<u32> {
    if n <= 100 { Some(n) } else { None }
}

fn and_then_example() {
    let input = "42";

    // parse_u32 で変換し、さらに check_max でフィルタ — 2 段の Option 処理
    let result = parse_u32(input).and_then(check_max);
    println!("{:?}", result); // Some(42)

    let too_big = parse_u32("200").and_then(check_max);
    println!("{:?}", too_big); // None

    let invalid = parse_u32("abc").and_then(check_max);
    println!("{:?}", invalid); // None
}
```

### `filter` — 条件でフィルタリングする

条件を満たさない `Some` を `None` に変換します。

```rust
fn filter_example() {
    let even: Option<u32> = Some(4).filter(|&n| n % 2 == 0);
    let odd: Option<u32>  = Some(3).filter(|&n| n % 2 == 0);

    println!("{:?}", even); // Some(4)
    println!("{:?}", odd);  // None
}
```

### `unwrap_or_else` — デフォルト値の遅延評価

`None` のときにクロージャを呼んでデフォルト値を生成します。コストが高い計算を `None` のときだけ実行したい場合に便利です。

```rust
fn unwrap_or_else_example() {
    let value: Option<String> = None;

    // None のときだけクロージャが呼ばれる（遅延評価）
    let result = value.unwrap_or_else(|| "デフォルト値".to_string());
    println!("{}", result); // デフォルト値

    let exists: Option<String> = Some("実際の値".to_string());
    let result2 = exists.unwrap_or_else(|| "デフォルト値".to_string());
    println!("{}", result2); // 実際の値（クロージャは呼ばれない）
}
```

### チェーンをつなぐ実例

```rust
fn option_chain_example(raw: &str) -> String {
    raw.trim()
        .parse::<u32>()
        .ok()                               // Result → Option
        .filter(|&n| n > 0)                 // 0 は除外
        .map(|n| n * n)                     // 2乗
        .map(|n| format!("結果: {}", n))    // 文字列に変換
        .unwrap_or_else(|| "無効な入力".to_string())
}

fn main() {
    println!("{}", option_chain_example("  9  ")); // 結果: 81
    println!("{}", option_chain_example("0"));     // 無効な入力
    println!("{}", option_chain_example("abc"));   // 無効な入力
}
```

---

## 6.5 `Result` の関数型チェーン

`Result` も `Option` と同様のメソッドを持ち、エラーを伝搬しながら変換チェーンを構築できます。

### `map` / `map_err` — 成功値とエラー値を変換する

```rust
use std::num::ParseIntError;

fn result_map_example() {
    let result: Result<i32, ParseIntError> = "10".parse::<i32>();

    // Ok の値を変換（Err はそのまま通過）
    let doubled: Result<i32, ParseIntError> = result.map(|n| n * 2);
    println!("{:?}", doubled); // Ok(20)

    let err_result: Result<i32, ParseIntError> = "abc".parse::<i32>();

    // Err の値を変換（Ok はそのまま通過）
    let mapped_err: Result<i32, String> = err_result.map_err(|e| format!("パースエラー: {}", e));
    println!("{:?}", mapped_err); // Err("パースエラー: ...")
}
```

### `and_then` — Result を返す処理を連鎖する

```rust
use std::num::ParseIntError;

/// 文字列を i32 にパース
fn parse(s: &str) -> Result<i32, ParseIntError> {
    s.parse::<i32>()
}

/// 正の数なら Ok、そうでなければ Err
fn ensure_positive(n: i32) -> Result<u32, String> {
    if n > 0 {
        Ok(n as u32)
    } else {
        Err(format!("{} は正の数ではありません", n))
    }
}

fn result_and_then_example() {
    // parse → ensure_positive の 2 段チェーン
    // エラー型が変わるため map_err でそろえる
    let success = parse("42")
        .map_err(|e| e.to_string())
        .and_then(ensure_positive);
    println!("{:?}", success); // Ok(42)

    let negative = parse("-5")
        .map_err(|e| e.to_string())
        .and_then(ensure_positive);
    println!("{:?}", negative); // Err("-5 は正の数ではありません")

    let invalid = parse("abc")
        .map_err(|e| e.to_string())
        .and_then(ensure_positive);
    println!("{:?}", invalid); // Err("invalid digit found in string") （ParseIntError のメッセージ）
}
```

### `unwrap_or_else` — エラー時のフォールバック

```rust
fn unwrap_or_else_result_example() {
    let result: Result<i32, &str> = Err("エラー発生");

    // Err のときだけクロージャが呼ばれる
    let value = result.unwrap_or_else(|e| {
        println!("回復処理: {}", e);
        -1 // デフォルト値
    });

    println!("値: {}", value); // 値: -1
}
```

---

## 6.6 `?` 演算子の仕組みと脱糖（desugaring）

`?` 演算子は `Result` と `Option` の早期リターンを簡潔に書くための構文糖衣です。

### `?` の脱糖

```rust
// ? を使った場合
fn with_question_mark(s: &str) -> Result<i32, String> {
    let n: i32 = s.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(n * 2)
}

// ? を展開すると以下と等価
fn without_question_mark(s: &str) -> Result<i32, String> {
    let n: i32 = match s.parse::<i32>().map_err(|e| e.to_string()) {
        Ok(val)  => val,
        Err(err) => return Err(err), // 即座に関数から Err を返す
    };
    Ok(n * 2)
}
```

`?` は内部で `From::from(err)` を呼び出します。これにより、エラー型が異なっていても `From` トレイトが実装されていれば自動変換が行われます。

### 複数の `?` を連鎖させる実例

```rust
use std::fs;
use std::num::ParseIntError;
use std::io;

#[derive(Debug)]
enum ReadError {
    Io(io::Error),
    Parse(ParseIntError),
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::Io(e)
    }
}

impl From<ParseIntError> for ReadError {
    fn from(e: ParseIntError) -> Self {
        ReadError::Parse(e)
    }
}

/// ファイルを読み込み、内容を整数にパースして 2 倍にする。
/// ファイル IO エラーとパースエラーの両方を `?` で処理する。
fn read_and_double(path: &str) -> Result<i32, ReadError> {
    let contents = fs::read_to_string(path)?; // io::Error → ReadError::Io（From 自動適用）
    let n: i32 = contents.trim().parse()?;    // ParseIntError → ReadError::Parse（From 自動適用）
    Ok(n * 2)
}
```

### `Option` での `?`

`?` は `Option` にも使えます。`None` のとき即座に `None` を返します。

```rust
/// 文字列スライスの最初の文字を数値として取得する
fn first_char_as_digit(s: &str) -> Option<u32> {
    let c = s.chars().next()?; // None なら即 None を返す
    c.to_digit(10)             // 数字でない場合も None
}

fn main() {
    println!("{:?}", first_char_as_digit("3abc")); // Some(3)
    println!("{:?}", first_char_as_digit("xyz"));  // None
    println!("{:?}", first_char_as_digit(""));     // None
}
```

---

## 6.7 カスタムエラー型の設計

実際のアプリケーションでは複数の種類のエラーが発生します。カスタムエラー型でそれらを統合します。

### `std::fmt::Display` と `std::error::Error` の実装

```rust
use std::fmt;
use std::num::ParseIntError;

/// アプリケーション全体のエラー型
#[derive(Debug)]
pub enum AppError {
    /// 文字列パースに失敗した
    ParseError(ParseIntError),
    /// 入力値が許容範囲外
    OutOfRange { value: i32, min: i32, max: i32 },
    /// 設定ファイルが見つからない
    ConfigNotFound(String),
}

// ユーザー向けのエラーメッセージ
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ParseError(e) => {
                write!(f, "パースエラー: {}", e)
            }
            AppError::OutOfRange { value, min, max } => {
                write!(f, "値 {} は範囲 [{}, {}] の外です", value, min, max)
            }
            AppError::ConfigNotFound(path) => {
                write!(f, "設定ファイルが見つかりません: {}", path)
            }
        }
    }
}

// std::error::Error を実装することで汎用的なエラートレイトとして扱える
impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::ParseError(e) => Some(e),
            _ => None,
        }
    }
}
```

---

## 6.8 `From` トレイトによるエラー変換

`From` トレイトを実装することで、`?` 演算子が自動的にエラー型を変換します。

```rust
use std::num::ParseIntError;

/// ParseIntError → AppError への変換
/// これにより parse::<i32>()?  が AppError を返す関数で使える
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::ParseError(e)
    }
}

/// 文字列を受け取り、範囲チェックまで行う
fn parse_and_validate(s: &str, min: i32, max: i32) -> Result<i32, AppError> {
    let n: i32 = s.parse()?; // ParseIntError は From により AppError::ParseError に変換

    if n < min || n > max {
        return Err(AppError::OutOfRange { value: n, min, max });
    }

    Ok(n)
}

fn main() {
    println!("{:?}", parse_and_validate("42", 0, 100));   // Ok(42)
    println!("{:?}", parse_and_validate("200", 0, 100));  // Err(OutOfRange { ... })
    println!("{:?}", parse_and_validate("abc", 0, 100));  // Err(ParseError(...))
}
```

### `thiserror` クレートによる簡略化

実際のプロジェクトでは `thiserror` クレートを使うと `Display` と `From` をマクロで自動生成できます（参考）。

```rust
// Cargo.toml に thiserror = "1" を追加した場合の例
// use thiserror::Error;
//
// #[derive(Debug, Error)]
// pub enum AppError {
//     #[error("パースエラー: {0}")]
//     ParseError(#[from] std::num::ParseIntError),
//
//     #[error("値 {value} は範囲 [{min}, {max}] の外です")]
//     OutOfRange { value: i32, min: i32, max: i32 },
// }
```

---

## 6.9 `Option` と `Result` の相互変換

`Option` と `Result` は相互に変換できます。

| メソッド | 変換 | 説明 |
|---------|------|------|
| `option.ok_or(err)` | `Option<T>` → `Result<T, E>` | `None` を指定したエラーに変換 |
| `option.ok_or_else(\|\| err)` | `Option<T>` → `Result<T, E>` | `None` をクロージャのエラーに変換 |
| `result.ok()` | `Result<T, E>` → `Option<T>` | `Err` を `None` に変換（エラー情報は破棄） |
| `result.err()` | `Result<T, E>` → `Option<E>` | `Ok` を `None` に変換 |
| `option_of_result.transpose()` | `Option<Result<T, E>>` → `Result<Option<T>, E>` | 型を入れ替える |
| `result_of_option.transpose()` | `Result<Option<T>, E>` → `Option<Result<T, E>>` | 型を入れ替える |

```rust
fn conversion_examples() {
    // Option → Result
    let opt: Option<i32> = Some(42);
    let res: Result<i32, &str> = opt.ok_or("値がありませんでした");
    println!("{:?}", res); // Ok(42)

    let none: Option<i32> = None;
    let res2: Result<i32, &str> = none.ok_or("値がありませんでした");
    println!("{:?}", res2); // Err("値がありませんでした")

    // Result → Option
    let ok: Result<i32, &str> = Ok(42);
    let opt2: Option<i32> = ok.ok();
    println!("{:?}", opt2); // Some(42)

    let err: Result<i32, &str> = Err("エラー");
    let opt3: Option<i32> = err.ok();
    println!("{:?}", opt3); // None（エラー情報は失われる）

    // transpose: Option<Result<T, E>> → Result<Option<T>, E>
    let some_ok: Option<Result<i32, &str>> = Some(Ok(42));
    let transposed: Result<Option<i32>, &str> = some_ok.transpose();
    println!("{:?}", transposed); // Ok(Some(42))

    let some_err: Option<Result<i32, &str>> = Some(Err("失敗"));
    let transposed2: Result<Option<i32>, &str> = some_err.transpose();
    println!("{:?}", transposed2); // Err("失敗")

    let none_val: Option<Result<i32, &str>> = None;
    let transposed3: Result<Option<i32>, &str> = none_val.transpose();
    println!("{:?}", transposed3); // Ok(None)
}
```

---

## 6.10 イテレータと `Result` の組み合わせ

イテレータで `Result` を扱う際、`collect::<Result<Vec<_>, _>>()` を使うと「1 つでも失敗があれば全体を `Err` にする」という処理が書けます。

```rust
use std::num::ParseIntError;

fn parse_numbers(inputs: &[&str]) -> Result<Vec<i32>, ParseIntError> {
    inputs
        .iter()
        .map(|s| s.parse::<i32>()) // Iterator<Item = Result<i32, ParseIntError>>
        .collect()                 // 全て Ok なら Ok(Vec<i32>)、1 つでも Err なら Err(...)
}

fn main() {
    let all_valid = parse_numbers(&["1", "2", "3"]);
    println!("{:?}", all_valid); // Ok([1, 2, 3])

    let has_invalid = parse_numbers(&["1", "abc", "3"]);
    println!("{:?}", has_invalid); // Err(ParseIntError { ... })
}
```

逆に「失敗を無視して成功した値だけを集める」場合は `filter_map` と `Result::ok` を組み合わせます。

```rust
fn parse_valid_only(inputs: &[&str]) -> Vec<i32> {
    inputs
        .iter()
        .filter_map(|s| s.parse::<i32>().ok()) // Err を None として除外
        .collect()
}

fn main() {
    let result = parse_valid_only(&["1", "abc", "3", "xyz", "5"]);
    println!("{:?}", result); // [1, 3, 5]
}
```

---

## 6.11 まとめ

| 目的 | 手段 |
|------|------|
| null の代替 | `Option<T>` |
| 例外の代替 | `Result<T, E>` |
| 値の変換 | `map` |
| Option/Result を返す処理の連鎖 | `and_then` |
| エラー値の変換 | `map_err` |
| 条件フィルタ | `filter`（Option のみ） |
| デフォルト値（遅延） | `unwrap_or_else` |
| 早期リターン | `?` 演算子 |
| 複数エラー型の統合 | カスタム enum + `From` トレイト |
| Option ↔ Result 変換 | `ok()`, `ok_or()`, `transpose()` |
| イテレータで全件検証 | `collect::<Result<Vec<_>, _>>()` |

`Option` と `Result` を使いこなすことで、エラー処理が型システムによって保証された「安全で読みやすいコード」になります。これが Rust の関数型エラーハンドリングの本質です。

---

## 章末演習問題

### 演習 1: `Option` チェーンの実装

以下の仕様を満たす関数 `find_double_digit` を実装してください。

- `&[&str]` を受け取る
- スライスの最初の要素を `u32` にパースする
- パースできた場合、値が 10 以上 99 以下（2 桁）であることを確認する
- 条件を満たせば値を 2 倍にして `Some` で返す、そうでなければ `None`
- `map`, `and_then`, `filter` を使ってチェーンで書くこと

```rust
fn find_double_digit(items: &[&str]) -> Option<u32> {
    // ここを実装してください
    todo!()
}
```

<details>
<summary>ヒント</summary>

`items.first()` でスライスの最初の要素（`Option<&&str>`）を取得できます。`and_then` でパース、`filter` で範囲チェック、`map` で 2 倍にします。

</details>

---

### 演習 2: カスタムエラー型と `?` 演算子

以下の 3 種類のエラーを持つ `CalcError` enum を定義してください。

1. `DivisionByZero` — 0 による除算
2. `ParseError(String)` — 文字列パース失敗（エラーメッセージを保持）
3. `Overflow` — 演算結果が `i32::MAX` を超えた

次に、以下の仕様の関数を `?` 演算子を使って実装してください。

```rust
/// 文字列 a と b をパースし、a を b で割った商を返す。
/// b が "0" のときは DivisionByZero、パース失敗は ParseError、
/// 商が i32::MAX を超える場合は Overflow を返す（今回のデータ範囲では Overflow は起きないが型として定義する）。
fn safe_divide(a: &str, b: &str) -> Result<i32, CalcError> {
    // ここを実装してください
    todo!()
}
```

---

### 演習 3: イテレータと `Result` の統合

以下の関数を実装してください。

```rust
/// 文字列スライスを受け取り、各要素を i32 にパースして合計を返す。
/// 1 つでもパースに失敗した場合は Err を返す（collect を使うこと）。
fn sum_strings(inputs: &[&str]) -> Result<i32, std::num::ParseIntError> {
    // ここを実装してください
    todo!()
}

/// 文字列スライスを受け取り、パースできたものだけの合計を返す。
/// パース失敗は無視する（filter_map を使うこと）。
fn sum_valid_strings(inputs: &[&str]) -> i32 {
    // ここを実装してください
    todo!()
}
```

`sum_strings(&["1", "2", "3"])` → `Ok(6)`
`sum_strings(&["1", "abc", "3"])` → `Err(...)`
`sum_valid_strings(&["1", "abc", "3", "xyz", "5"])` → `9`

---

## 強化: anyhow / thiserror の実用的な使い方

### 使い分けの原則

| ライブラリ | 用途 | 場面 |
|-----------|------|------|
| `thiserror` | ドメインエラーの定義 | ライブラリ・コアロジック |
| `anyhow` | エラーの伝播・集約 | アプリケーション層・`main()` |

```toml
[dependencies]
thiserror = "2"
anyhow = "1"
```

### thiserror によるエラー定義

`#[derive(Error)]` と `#[error("...")]` でエラーメッセージを宣言的に記述できます。

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("ユーザーが見つかりません: id={0}")]
    NotFound(u32),

    #[error("メールアドレスの形式が不正です: {email}")]
    InvalidEmail { email: String },

    #[error("パスワードが短すぎます: {len}文字 (最低{min}文字必要)")]
    PasswordTooShort { len: usize, min: usize },
}
```

#### `#[from]` でエラーの自動変換

```rust
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("ユーザーエラー: {0}")]
    User(#[from] UserError),   // UserError → ServiceError が自動実装

    #[error("インフラエラー: {0}")]
    Infra(#[from] InfraError),
}

// ? 演算子で自動変換
fn service_fn() -> Result<(), ServiceError> {
    validate_email("bad")?;  // UserError が ServiceError に変換される
    Ok(())
}
```

### anyhow によるエラー処理

`anyhow::Result<T>` は `Result<T, anyhow::Error>` の型エイリアスで、どんなエラー型も受け取れます。

```rust
use anyhow::{Context, Result, bail, ensure};

fn parse_port(s: &str) -> Result<u16> {
    // .context() でエラーにコンテキストを追加
    let port: u16 = s.parse().context("ポート番号は数値で指定してください")?;

    // ensure! で条件チェック（偽なら Err を返す）
    ensure!(port > 0, "ポート番号は1以上である必要があります");

    Ok(port)
}

fn check_age(age: i32) -> Result<()> {
    if age < 0 {
        // bail! で即 Err を返す
        bail!("年齢は0以上でなければなりません: {}", age);
    }
    Ok(())
}
```

### 組み合わせパターン

ドメイン層で `thiserror`、アプリケーション層で `anyhow` を使うのが実践的です。

```rust
use anyhow::{Context, Result};

fn create_user(email: &str, password: &str) -> Result<String> {
    // thiserror のエラーを anyhow でラップ
    validate_email(email)
        .context("メールアドレスのバリデーションに失敗しました")?;

    validate_password(password)
        .context("パスワードのバリデーションに失敗しました")?;

    Ok(format!("ユーザー作成成功: {}", email))
}
```

エラーチェーンが自動で作られるため、デバッグ時にエラーの根本原因を追跡しやすくなります。
