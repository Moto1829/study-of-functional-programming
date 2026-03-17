# イテレータ（Iterators）

## 概要

Rustのイテレータは**遅延評価（Lazy Evaluation）**を採用しており、値が実際に必要になるまで計算を行いません。`map`、`filter`、`flat_map` などのアダプタをチェーンして、宣言的なデータ処理パイプラインを構築できます。

## 基本的な使い方

```rust
let numbers = vec![1, 2, 3, 4, 5];

// iter(): 不変参照のイテレータ
let sum: i32 = numbers.iter().sum();

// into_iter(): 所有権を消費するイテレータ
let doubled: Vec<i32> = numbers.iter().map(|&n| n * 2).collect();
```

## 主なイテレータアダプタ

### `map` — 変換
```rust
let squares: Vec<i32> = (1..=5).map(|n| n * n).collect();
// [1, 4, 9, 16, 25]
```

### `filter` — 絞り込み
```rust
let evens: Vec<i32> = (1..=10).filter(|n| n % 2 == 0).collect();
// [2, 4, 6, 8, 10]
```

### `flat_map` — 変換＋平坦化
```rust
let words = vec!["hello world", "foo bar"];
let chars: Vec<&str> = words.iter().flat_map(|s| s.split(' ')).collect();
// ["hello", "world", "foo", "bar"]
```

### `zip` — 2つのイテレータを組み合わせる
```rust
let a = vec![1, 2, 3];
let b = vec!["one", "two", "three"];
let zipped: Vec<_> = a.iter().zip(b.iter()).collect();
// [(1, "one"), (2, "two"), (3, "three")]
```

### `take` と `skip`
```rust
let first_three: Vec<i32> = (1..=10).take(3).collect(); // [1, 2, 3]
let skip_two: Vec<i32> = (1..=5).skip(2).collect();     // [3, 4, 5]
```

## カスタムイテレータ

`Iterator` トレイトを実装することで、独自のイテレータを作れます：

```rust
pub struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    pub fn new() -> Self {
        Fibonacci { a: 0, b: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(self.a)
    }
}

// 最初の10個のフィボナッチ数
let fibs: Vec<u64> = Fibonacci::new().take(10).collect();

// 偶数フィボナッチ数の最初の5つ（無限イテレータを遅延評価で処理）
let even_fibs: Vec<u64> = Fibonacci::new()
    .filter(|n| n % 2 == 0)
    .take(5)
    .collect();
```

## 遅延評価の力

```rust
// これは実際には何も計算しない（遅延評価）
let lazy = (1..).map(|x| x * x).filter(|&x| x % 2 == 0);

// collect() や sum() で初めて評価される
let first_five_even_squares: Vec<u64> = lazy.take(5).collect();
// [4, 16, 36, 64, 100]
```

## 演習

`crates/iterators/src/main.rs` を参照してください。

```bash
cargo run -p iterators
cargo test -p iterators
```
