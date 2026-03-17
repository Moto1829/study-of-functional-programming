# Option と Result

## 概要

Rustの `Option<T>` と `Result<T, E>` は、関数型プログラミングにおける**モナド（Monad）**的な型です。`null` 参照やエラーを型安全に表現し、`map`、`and_then`、`unwrap_or` などのメソッドでチェーンして使います。

## Option\<T\>

`Option<T>` は「値があるかもしれない、ないかもしれない」を表します：

```rust
enum Option<T> {
    Some(T),   // 値がある
    None,      // 値がない
}
```

### 例: 安全な除算

```rust
fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

// match でパターンマッチング
match safe_divide(10.0, 2.0) {
    Some(result) => println!("結果: {}", result),
    None => println!("ゼロ除算エラー"),
}

// unwrap_or でデフォルト値
let result = safe_divide(10.0, 0.0).unwrap_or(0.0); // 0.0
```

### Option のメソッドチェーン

```rust
// 文字列をパースして偶数かチェック
fn parse_even(s: &str) -> Option<i32> {
    s.trim()
        .parse::<i32>()
        .ok()                      // Result -> Option
        .filter(|&n| n % 2 == 0)  // 偶数のみ
}
```

## Result\<T, E\>

`Result<T, E>` は「成功か失敗か」を表します：

```rust
enum Result<T, E> {
    Ok(T),   // 成功: 値 T
    Err(E),  // 失敗: エラー E
}
```

### 例: エラーハンドリング

```rust
fn parse_positive(s: &str) -> Result<u32, String> {
    let n: i32 = s.trim()
        .parse()
        .map_err(|_| format!("'{}' は整数ではありません", s))?;

    if n < 0 {
        Err(format!("{} は負の値です", n))
    } else {
        Ok(n as u32)
    }
}
```

### `?` 演算子

`?` 演算子を使うと、エラーを早期リターンできます：

```rust
fn parse_and_add(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x: i32 = a.trim().parse()?;  // エラーなら即座にリターン
    let y: i32 = b.trim().parse()?;  // エラーなら即座にリターン
    Ok(x + y)
}
```

## Result のメソッドチェーン

```rust
let result = "42"
    .parse::<i32>()
    .map(|n| n * 2)          // Ok の場合だけ変換
    .map_err(|e| e.to_string()); // Err の場合だけ変換

println!("{:?}", result); // Ok(84)
```

## Option と Result の比較

| 型 | 意味 | 主な用途 |
|----|------|---------|
| `Option<T>` | 値があるかもしれない | 検索、任意の設定値 |
| `Result<T, E>` | 成功か失敗か | I/O、パース、外部API |

## `map`, `and_then` の活用

```rust
// Option をチェーン
let result: Option<String> = "5"
    .parse::<i32>()
    .ok()
    .filter(|&n| n > 0)
    .map(|n| n * 2)
    .map(|n| format!("結果: {}", n));
// Some("結果: 10")
```

## 演習

`crates/option_and_result/src/main.rs` を参照してください。

```bash
cargo run -p option_and_result
cargo test -p option_and_result
```
