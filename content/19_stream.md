# 第19章: Stream / 非同期イテレータ

## はじめに

第4章ではイテレータを学びました。イテレータは「同期的に値を一つずつ生成する」仕組みです。しかし現実のデータは非同期に到着することが多いです（ネットワーク、ファイル I/O など）。

**Stream** は「非同期版イテレータ」です。`Iterator` が `next() -> Option<T>` を返すのに対し、`Stream` は `next() -> Future<Output = Option<T>>` を返します。

```
Iterator: 同期、即座に次の値が得られる
Stream:   非同期、次の値は将来到着する
```

---

## Rust における Stream の基礎

Rust の標準ライブラリには現在 Stream が安定化されていません（2026年時点）。実用的には `futures` クレートの `Stream` トレイト、または `tokio` の `tokio_stream` クレートを使います。

しかし、Stream の本質は**同期イテレータと同じ関数型パターン**を非同期に適用することです。本章では以下を扱います：

1. `async fn` と `Future` の基礎
2. 同期 Stream（独自実装）で Stream の概念を学ぶ
3. `futures` クレートを使った非同期 Stream
4. 関数型スタイル: `map`、`filter`、`fold` の非同期版

---

## async / await の基礎

`async fn` は `Future` を返す関数です：

```rust
async fn fetch_number(n: u32) -> u32 {
    // 実際はここでネットワーク通信などを行う
    n * 2
}

#[tokio::main]
async fn main() {
    let result = fetch_number(5).await;
    println!("{}", result); // 10
}
```

`async` ブロックの中では `.await` で Future の完了を待てます。

---

## 同期 Stream: Pull ベースの設計

まず同期で Stream の概念を理解します。Stream は「繰り返し `poll` されることで値を生成する」プル型の仕組みです：

```rust
pub trait Stream {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

これは `Iterator` とほぼ同じです。非同期版との違いは `next` が `Future` を返す点だけです。

### カスタム Stream の実装

```rust
/// 指定した範囲の整数を生成する Stream
pub struct RangeStream {
    current: u64,
    end: u64,
}

impl RangeStream {
    pub fn new(start: u64, end: u64) -> Self {
        RangeStream { current: start, end }
    }
}

impl Stream for RangeStream {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let val = self.current;
            self.current += 1;
            Some(val)
        } else {
            None
        }
    }
}
```

---

## Stream コンビネータ

イテレータと同様に、`map`、`filter`、`take`、`fold` などのコンビネータを Stream に定義できます：

```rust
/// map コンビネータ
pub struct MapStream<S, F> {
    inner: S,
    f: F,
}

impl<S: Stream, B, F: FnMut(S::Item) -> B> Stream for MapStream<S, F> {
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| (self.f)(item))
    }
}

/// filter コンビネータ
pub struct FilterStream<S, F> {
    inner: S,
    predicate: F,
}

impl<S: Stream, F: FnMut(&S::Item) -> bool> Stream for FilterStream<S, F> {
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                None => return None,
                Some(item) if (self.predicate)(&item) => return Some(item),
                Some(_) => continue,
            }
        }
    }
}

/// take コンビネータ
pub struct TakeStream<S> {
    inner: S,
    remaining: usize,
}

impl<S: Stream> Stream for TakeStream<S> {
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        self.inner.next()
    }
}
```

### Stream に流暢な API を追加する

トレイトを拡張してメソッドチェーンを可能にします：

```rust
pub trait StreamExt: Stream + Sized {
    fn map_stream<B, F: FnMut(Self::Item) -> B>(self, f: F) -> MapStream<Self, F> {
        MapStream { inner: self, f }
    }

    fn filter_stream<F: FnMut(&Self::Item) -> bool>(self, predicate: F) -> FilterStream<Self, F> {
        FilterStream { inner: self, predicate }
    }

    fn take_stream(self, n: usize) -> TakeStream<Self> {
        TakeStream { inner: self, remaining: n }
    }

    fn fold_stream<B, F: FnMut(B, Self::Item) -> B>(mut self, init: B, mut f: F) -> B {
        let mut acc = init;
        while let Some(item) = self.next() {
            acc = f(acc, item);
        }
        acc
    }

    fn collect_stream(mut self) -> Vec<Self::Item> {
        let mut result = Vec::new();
        while let Some(item) = self.next() {
            result.push(item);
        }
        result
    }
}

impl<S: Stream + Sized> StreamExt for S {}
```

---

## 関数型パイプラインとして使う

```rust
fn main() {
    // 0..100 の中から偶数を選び、2倍して、最初の5件を合計する
    let result = RangeStream::new(0, 100)
        .filter_stream(|n| n % 2 == 0)
        .map_stream(|n| n * 2)
        .take_stream(5)
        .fold_stream(0, |acc, n| acc + n);

    println!("{}", result); // 0+4+8+12+16 = 40
}
```

これはイテレータと全く同じパターンです。非同期 Stream でも同じコンビネータが使えます。

---

## 無限 Stream

Stream はイテレータと同様に無限のシーケンスを表現できます：

```rust
/// フィボナッチ数列を無限に生成する Stream
pub struct FibStream {
    a: u64,
    b: u64,
}

impl FibStream {
    pub fn new() -> Self {
        FibStream { a: 0, b: 1 }
    }
}

impl Stream for FibStream {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.a;
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(result) // 無限に Some を返す
    }
}

fn main() {
    let first_10: Vec<u64> = FibStream::new()
        .take_stream(10)
        .collect_stream();

    println!("{:?}", first_10);
    // [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
}
```

---

## `futures` クレートでの非同期 Stream

実際の非同期プログラミングでは `futures::stream` を使います：

```toml
[dependencies]
futures = "0.3"
tokio = { version = "1", features = ["full"] }
```

```rust
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() {
    // 同期的な値から Stream を作成
    let sum = stream::iter(0u32..10)
        .filter(|n| futures::future::ready(n % 2 == 0))
        .map(|n| n * n)
        .fold(0u32, |acc, n| async move { acc + n })
        .await;

    println!("{}", sum); // 0+4+16+36+64 = 120
}
```

`stream::iter` で任意のイテレータを非同期 Stream に変換できます。コンビネータ名は同期版とほぼ同じですが、各コンビネータが `Future` を返します。

### 非同期処理の並列実行

```rust
use futures::stream::{self, StreamExt};

#[tokio::main]
async fn main() {
    // 複数の非同期処理を並列に実行して結果を収集
    let results: Vec<u32> = stream::iter(0u32..5)
        .map(|n| async move {
            // tokio::time::sleep(Duration::from_millis(100)).await;
            n * n
        })
        .buffer_unordered(3) // 最大3つ並行実行
        .collect()
        .await;

    println!("{:?}", results);
}
```

`buffer_unordered(n)` は最大 `n` 個の Future を並行して実行します。これが非同期 Stream の強力な点です。

---

## チャネルを Stream として使う

`tokio::sync::mpsc` チャネルは非同期 Stream として使えます：

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<i32>(32);

    // プロデューサー
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });

    // コンシューマー: 関数型スタイルで処理
    let mut sum = 0;
    while let Some(value) = rx.recv().await {
        sum += value;
    }
    println!("合計: {}", sum); // 10
}
```

---

## まとめ

| 概念 | 同期版 | 非同期版 |
|------|--------|---------|
| 基本型 | `Iterator` | `Stream` |
| 次の値を取得 | `next() -> Option<T>` | `next() -> Future<Output = Option<T>>` |
| 変換 | `.map(f)` | `.map(f)` |
| フィルタ | `.filter(pred)` | `.filter(pred)` |
| 集計 | `.fold(init, f)` | `.fold(init, f).await` |
| 並列実行 | なし（rayon） | `.buffer_unordered(n)` |

Stream は「**時間軸上に広がったイテレータ**」です。関数型のコンビネータがそのまま非同期世界に拡張されます。

---

## よくある落とし穴と対処法

**落とし穴1: `Stream` を `send` できない**

非同期クロージャ内で `Arc<Mutex<T>>` を使わず普通の参照を持ち込むと `Send` トレイトが満たせずコンパイルエラーになります。クロージャの外部変数は `Arc` でラップしましょう。

**落とし穴2: `buffer_unordered` での順序**

`buffer_unordered` は完了順に結果を返すため、入力の順序と出力の順序が異なります。順序を保持したい場合は `buffered` を使います。

**落とし穴3: チャネルの `drop` 忘れ**

プロデューサー側の送信者 (`tx`) を `drop` しないと、コンシューマーが永遠に次の値を待ち続けます（第11章参照）。

---

## 章末演習問題

1. `RangeStream` を使って「1から100の中で3の倍数かつ5の倍数の合計」を計算してください（`filter_stream` と `fold_stream` を使う）。

2. 「前の値と現在の値をペアにして返す Stream」（`windows(2)` 相当）を実装してください：
```rust
// 入力: 1, 2, 3, 4, 5
// 出力: (1,2), (2,3), (3,4), (4,5)
```

3. `FibStream` と `RangeStream` を使って「フィボナッチ数列の中から偶数番目（0-indexed）の値だけを10個取り出す」コードを書いてください。
