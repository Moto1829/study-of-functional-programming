# 第17章: Free Monad

## はじめに

**Free Monad** は「何をするかの記述（DSL）」と「実際の実行」を分離するパターンです。

例えばデータベース操作を考えます。通常はコードの中に直接 SQL を実行しますが、Free Monad を使うと「操作の列」を純粋なデータとして構築し、後から「どう実行するか」を注入できます。

```
プログラムが書くもの: 操作の記述（純粋なデータ）
                        ↓
インタープリタ:      実際の実行（本番DB、テスト用モック、ログ出力など）
```

これにより**テスタビリティ**と**関心の分離**が大幅に向上します。

---

## Free Monad の構造

Free Monad は次の2つのバリアントを持つ enum です：

```rust
enum Free<F, A> {
    Pure(A),                    // 完了した値
    Free(Box<F<Free<F, A>>>),   // 次のステップを含む操作
}
```

Rust の型システムでは高カインド型（Higher-Kinded Types, HKT）がないため、完全な汎用実装は難しいです。実用上は**特定の DSL に特化した Free Monad** を作るアプローチが現実的です。

---

## ストレージ DSL の例

「キー・バリューストアへの操作」を DSL として定義します。

```rust
/// ストレージ操作の DSL
pub enum StorageOp<Next> {
    Get(String, Box<dyn FnOnce(Option<String>) -> Next>),
    Set(String, String, Box<dyn FnOnce() -> Next>),
    Delete(String, Box<dyn FnOnce() -> Next>),
}

/// Free Monad: 操作の連鎖を表すデータ構造
pub enum Program<A> {
    Pure(A),
    Step(StorageOp<Program<A>>),
}
```

`Program<A>` は「最終的に `A` を返すストレージ操作のシーケンス」を純粋なデータとして表現します。

---

## スマートコンストラクタ

操作を構築するヘルパー関数（スマートコンストラクタ）を定義します：

```rust
pub fn get(key: impl Into<String>) -> Program<Option<String>> {
    Program::Step(StorageOp::Get(
        key.into(),
        Box::new(Program::Pure),
    ))
}

pub fn set(key: impl Into<String>, value: impl Into<String>) -> Program<()> {
    Program::Step(StorageOp::Set(
        key.into(),
        value.into(),
        Box::new(|| Program::Pure(())),
    ))
}

pub fn delete(key: impl Into<String>) -> Program<()> {
    Program::Step(StorageOp::Delete(
        key.into(),
        Box::new(|| Program::Pure(())),
    ))
}
```

---

## `and_then` による操作の連鎖

`Program` に `and_then` を実装することで、操作を連鎖させられます：

```rust
impl<A: 'static> Program<A> {
    pub fn and_then<B: 'static, F>(self, f: F) -> Program<B>
    where
        F: FnOnce(A) -> Program<B> + 'static,
    {
        match self {
            Program::Pure(a) => f(a),
            Program::Step(op) => Program::Step(match op {
                StorageOp::Get(k, next) => StorageOp::Get(
                    k,
                    Box::new(move |v| next(v).and_then(f)),
                ),
                StorageOp::Set(k, v, next) => StorageOp::Set(
                    k, v,
                    Box::new(move || next().and_then(f)),
                ),
                StorageOp::Delete(k, next) => StorageOp::Delete(
                    k,
                    Box::new(move || next().and_then(f)),
                ),
            }),
        }
    }

    pub fn map<B: 'static, F>(self, f: F) -> Program<B>
    where
        F: FnOnce(A) -> B + 'static,
    {
        self.and_then(move |a| Program::Pure(f(a)))
    }
}
```

---

## DSL でプログラムを記述する

スマートコンストラクタと `and_then` を使って、副作用を持たない純粋なデータとしてプログラムを記述します：

```rust
fn transfer_value(from: &str, to: &str) -> Program<bool> {
    get(from).and_then(move |from_val| {
        match from_val {
            None => Program::Pure(false),
            Some(val) => {
                set(to, val.clone())
                    .and_then(move |_| delete(from))
                    .and_then(|_| Program::Pure(true))
            }
        }
    })
}
```

この関数は**何もしていません**。ただ「何をするかの記述」を返しているだけです。

---

## インタープリタ: 本番実装

実際の実行はインタープリタが担います。同じ `Program` に対して異なるインタープリタを差し替えられます。

```rust
use std::collections::HashMap;

// 本番用: 実際の HashMap で実行
pub fn run_in_memory(program: Program<bool>, store: &mut HashMap<String, String>) -> bool {
    match program {
        Program::Pure(a) => a,
        Program::Step(op) => match op {
            StorageOp::Get(k, next) => {
                let v = store.get(&k).cloned();
                run_in_memory(next(v), store)
            }
            StorageOp::Set(k, v, next) => {
                store.insert(k, v);
                run_in_memory(next(), store)
            }
            StorageOp::Delete(k, next) => {
                store.remove(&k);
                run_in_memory(next(), store)
            }
        },
    }
}
```

---

## インタープリタ: テスト用 (ログ付き)

```rust
pub fn run_with_log(
    program: Program<bool>,
    store: &mut HashMap<String, String>,
    log: &mut Vec<String>,
) -> bool {
    match program {
        Program::Pure(a) => a,
        Program::Step(op) => match op {
            StorageOp::Get(k, next) => {
                let v = store.get(&k).cloned();
                log.push(format!("GET {} → {:?}", k, v));
                run_with_log(next(v), store, log)
            }
            StorageOp::Set(k, v, next) => {
                log.push(format!("SET {} = {}", k, v));
                store.insert(k, v);
                run_with_log(next(), store, log)
            }
            StorageOp::Delete(k, next) => {
                log.push(format!("DELETE {}", k));
                store.remove(&k);
                run_with_log(next(), store, log)
            }
        },
    }
}
```

---

## 使用例

```rust
fn main() {
    let mut store = HashMap::new();
    store.insert("from".to_string(), "hello".to_string());

    // プログラムの記述（副作用なし）
    let prog = transfer_value("from", "to");

    // 実行（インタープリタに委ねる）
    let mut log = vec![];
    let success = run_with_log(prog, &mut store, &mut log);

    println!("成功: {}", success);        // true
    println!("from: {:?}", store.get("from")); // None（削除済み）
    println!("to: {:?}", store.get("to"));     // Some("hello")

    println!("\n実行ログ:");
    for entry in &log {
        println!("  {}", entry);
    }
    // GET from → Some("hello")
    // SET to = hello
    // DELETE from
}
```

---

## Free Monad の利点まとめ

| 利点 | 説明 |
|------|------|
| **テスタビリティ** | モックインタープリタを差し替えてテストできる |
| **関心の分離** | 「何をするか」と「どう実行するか」が分離する |
| **複数の解釈** | 同じプログラムをDBで実行、ログ記録、最適化など異なる方法で実行できる |
| **純粋性** | プログラム記述部分は副作用がなく、推論しやすい |

---

## よくある落とし穴と対処法

**落とし穴1: スタックオーバーフロー**

深い `and_then` の連鎖はスタックを消費します。第10章で学んだ Trampoline パターンと組み合わせることで解決できます。

**落とし穴2: パフォーマンス**

Free Monad は各ステップで `Box` を使うため、パフォーマンスが重要な場面では直接実装の方が良いことがあります。Free Monad は**設計の明確さ**と**テスタビリティ**を優先する場面で使いましょう。

**落とし穴3: 複雑な DSL の合成**

複数の DSL を合成するには `Coproduct`（和型の合成）が必要になり、Rust では複雑になります。実用上は操作を一つの enum にまとめるシンプルなアプローチを取ることが多いです。

---

## 章末演習問題

1. `StorageOp` に `GetAll` 操作（全キーのリストを返す）を追加し、スマートコンストラクタとインタープリタを実装してください。

2. 以下のコンソール操作 DSL を実装してください：
```rust
enum ConsoleOp<Next> {
    Print(String, Box<dyn FnOnce() -> Next>),
    ReadLine(Box<dyn FnOnce(String) -> Next>),
}
```
テスト用インタープリタとして、事前に入力文字列を設定できる `run_mock(program, inputs: Vec<String>)` を実装してください。

3. `transfer_value` を `run_in_memory` と `run_with_log` の2つのインタープリタで実行し、結果が同じになることをテストで確認してください。
