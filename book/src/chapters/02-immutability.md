# 不変性（Immutability）

## 概要

**不変性**とは、一度作成されたデータを変更しないという概念です。関数型プログラミングでは、データを変更する代わりに新しいデータを生成します。

## Rust のデフォルト不変性

Rust では、変数は**デフォルトで不変（immutable）**です：

```rust
let x = 5;
// x = 6; // コンパイルエラー！

let mut y = 5;
y = 6; // これは OK（明示的に可変にしている）
```

## データ変換の例

関数型スタイルでは、データを「変更」するのではなく「変換」します：

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    // 元の点を変更せず、移動した新しい点を返す
    pub fn translate(&self, dx: f64, dy: f64) -> Point {
        Point {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

let origin = Point { x: 0.0, y: 0.0 };
let moved = origin.translate(3.0, 4.0);

println!("{:?}", origin); // 変化していない: Point { x: 0.0, y: 0.0 }
println!("{:?}", moved);  // 新しい値: Point { x: 3.0, y: 4.0 }
```

## リストの変換

```rust
let numbers = vec![1, 2, 3, 4, 5];

// map で新しい Vec を生成（元の Vec は変化しない）
let doubled: Vec<i32> = numbers.iter().map(|&n| n * 2).collect();

println!("{:?}", numbers); // [1, 2, 3, 4, 5] - 変化していない
println!("{:?}", doubled); // [2, 4, 6, 8, 10]
```

## なぜ不変性が重要なのか？

- **予測可能性**: データがいつ変わるか分からない問題がなくなる
- **スレッド安全**: 共有状態がないので、データ競合が発生しない
- **デバッグが簡単**: 値が変わらないのでバグの原因特定が容易
- **テストが容易**: 状態を初期化する必要がない

## 演習

`crates/immutability/src/main.rs` を参照してください。

```bash
cargo run -p immutability
cargo test -p immutability
```
