# 純粋関数（Pure Functions）

## 概要

**純粋関数**とは、以下の2つの性質を持つ関数です：

1. **参照透過性（Referential Transparency）**: 同じ引数を与えると必ず同じ結果を返す
2. **副作用なし（No Side Effects）**: グローバル変数の変更、I/O処理、状態の変更などを行わない

## なぜ重要なのか？

純粋関数には以下のメリットがあります：

- **テストが簡単**: 入出力のみをテストすればよい
- **デバッグが容易**: 状態に依存しないため、動作が予測しやすい
- **並列処理に適している**: 共有状態がないため、安全に並列実行できる
- **キャッシュ可能（メモ化）**: 同じ入力なら同じ結果が保証される

## Rust での例

```rust
// 純粋関数: 同じ入力 → 常に同じ出力、副作用なし
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// 円の面積（副作用なし、参照透過）
pub fn circle_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}

// 再帰的な純粋関数
pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

## 非純粋関数との比較

```rust
// 非純粋関数の例（状態に依存する）
static mut COUNTER: i32 = 0;

fn impure_add(a: i32) -> i32 {
    unsafe {
        COUNTER += 1;  // 副作用！
    }
    a + 1
}

// 純粋関数の例
fn pure_add_one(a: i32) -> i32 {
    a + 1  // 副作用なし
}
```

## 演習

`crates/pure_functions/src/main.rs` を参照してください。

```bash
cargo run -p pure_functions
cargo test -p pure_functions
```

## ポイント

- Rustでは、ほとんどの関数はデフォルトで純粋関数として書けます
- `&mut` や `static mut`、I/O を使う関数は副作用を持つ可能性があります
- 純粋関数は積極的に利用し、副作用は必要最小限に留めましょう
