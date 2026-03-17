# 第11章: 並行処理と関数型スタイル

## はじめに

関数型プログラミングの**不変性**と**副作用の分離**は、並行処理と相性が良いです。Rust の所有権システムは、コンパイル時にデータ競合を防ぎ、安全な並行処理を実現します。

---

## Arc による不変データの共有

`Arc<T>`（Atomic Reference Counted）を使うと、不変データを複数スレッドで安全に共有できます。

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3, 4, 5]);
let mut handles = vec![];

for i in 0..3 {
    let data = Arc::clone(&data);
    let handle = thread::spawn(move || data[i]);
    handles.push(handle);
}

let results: Vec<i32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
```

**ポイント:**
- `Arc::clone` はポインタをコピーするだけで、データはコピーしない
- `T` が不変なら `Arc<T>` だけで安全（`Mutex` 不要）
- 関数型スタイルと相性が良い：共有データを変更しない

---

## チャネルによるメッセージパッシング

Rust の `mpsc`（multiple producer, single consumer）チャネルはメッセージパッシングの基本です。

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

let values = vec![1, 2, 3, 4, 5];
for v in values {
    let tx = tx.clone();
    thread::spawn(move || {
        tx.send(v).unwrap();
    });
}
drop(tx); // 元の tx を drop しないと rx がブロックし続ける

let sum: i32 = rx.into_iter().sum();
```

チャネルは**副作用を局所化**する関数型のアプローチと相性が良く、スレッド間の通信を明示的にします。

---

## Mutex と関数型スタイル

可変状態が必要な場合は `Arc<Mutex<T>>` を使います。

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0i32));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        let mut c = counter.lock().unwrap();
        *c += 1;
    });
    handles.push(handle);
}

for h in handles { h.join().unwrap(); }

let result = *counter.lock().unwrap();
assert_eq!(result, 10);
```

**関数型スタイルのポイント:**
- `Mutex` の使用箇所を最小限に絞る
- ロック中は短い操作のみ行う
- できるだけ不変データ（`Arc` のみ）を優先する

---

## 不変設定データの共有パターン

設定やコンテキストなど、初期化後に変更しないデータは `Arc<T>` で共有するのが関数型スタイルです。

```rust
#[derive(Debug, Clone)]
struct Config {
    max_connections: u32,
    timeout_ms: u64,
    host: String,
}

let config = Arc::new(Config {
    max_connections: 100,
    timeout_ms: 5000,
    host: "localhost".to_string(),
});

let handles: Vec<_> = (0..4).map(|i| {
    let cfg = Arc::clone(&config);
    thread::spawn(move || {
        // config を参照するが変更しない
        format!("Task {} on {}", i, cfg.host)
    })
}).collect();
```

---

## 並列処理結果の fold による集約

複数スレッドの部分結果を `fold` で集約するパターンは関数型らしい書き方です。

```rust
let chunks = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];

let handles: Vec<_> = chunks
    .into_iter()
    .map(|chunk| thread::spawn(move || chunk.into_iter().sum::<i32>()))
    .collect();

let total = handles
    .into_iter()
    .map(|h| h.join().unwrap())
    .fold(0, |acc, x| acc + x);

assert_eq!(total, 45);
```

---

## Rust の並行安全性の保証

Rust はコンパイル時に以下を保証します：

| トレイト | 意味 |
|---------|------|
| `Send` | 型をスレッド間で送れる |
| `Sync` | 型への参照をスレッド間で共有できる |

`Arc<T>` は `T: Send + Sync` のとき `Send + Sync` になります。`Mutex<T>` は `T: Send` のとき `Sync` になります。

---

## まとめ

| 手法 | 用途 | 関数型との相性 |
|------|------|--------------|
| `Arc<T>` | 不変データの共有 | ◎ 副作用なし |
| `mpsc::channel` | スレッド間通信 | ◎ メッセージパッシング |
| `Arc<Mutex<T>>` | 可変状態の共有 | △ 最小限に留める |

関数型スタイルでは**不変データを優先**し、可変状態は `Mutex` で局所化することで、安全で理解しやすい並行コードが書けます。
