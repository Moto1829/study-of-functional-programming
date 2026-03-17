# 第10章: 再帰と末尾呼び出し

## はじめに

関数型プログラミングでは、ループの代わりに**再帰**を使うことが多いです。しかし単純な再帰はスタックオーバーフローのリスクがあります。本章では再帰の基本から、スタックを節約する手法までを解説します。

---

## 基本的な再帰

再帰とは、関数が自分自身を呼び出すことです。フィボナッチ数列で見てみましょう。

```rust
fn fib_naive(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib_naive(n - 1) + fib_naive(n - 2),
    }
}
```

**問題点:** この実装は呼び出しのたびにスタックフレームが積まれ、`n` が大きいとスタックオーバーフローが起きます。また計算量も指数的になります。

---

## 末尾再帰スタイル（累積引数パターン）

**末尾呼び出し**とは、関数の最後の操作が再帰呼び出しであることです。累積引数（accumulator）を使って末尾再帰スタイルに変換できます。

```rust
fn fib_tail(n: u64) -> u64 {
    fn go(n: u64, a: u64, b: u64) -> u64 {
        match n {
            0 => a,
            _ => go(n - 1, b, a + b),  // 末尾呼び出し
        }
    }
    go(n, 0, 1)
}
```

累積引数 `a`, `b` に状態を持たせることで、再帰後に計算が不要になります。

### 階乗の例

```rust
fn factorial_tail(n: u64) -> u64 {
    fn go(n: u64, acc: u64) -> u64 {
        match n {
            0 | 1 => acc,
            _ => go(n - 1, n * acc),  // 末尾呼び出し
        }
    }
    go(n, 1)
}
```

> **注意:** Rust は現時点で末尾呼び出し最適化（TCO）を保証していません。ただし、最適化ビルド時に末尾再帰がループに変換される場合があります。保証が必要な場合は後述の Trampoline パターンを使いましょう。

---

## Trampoline パターン

Trampoline は「次の計算」を遅延させることで、スタックを使わずに再帰を実現するテクニックです。

```rust
pub enum Trampoline<T> {
    Done(T),
    More(Box<dyn FnOnce() -> Trampoline<T>>),
}

impl<T> Trampoline<T> {
    pub fn run(self) -> T {
        let mut current = self;
        loop {
            match current {
                Trampoline::Done(value) => return value,
                Trampoline::More(thunk) => current = thunk(),
            }
        }
    }
}

fn factorial_trampoline(n: u64) -> u64 {
    fn go(n: u64, acc: u64) -> Trampoline<u64> {
        match n {
            0 | 1 => Trampoline::Done(acc),
            _ => Trampoline::More(Box::new(move || go(n - 1, n * acc))),
        }
    }
    go(n, 1).run()
}
```

`run()` メソッドはループで動作するため、どれだけ深い再帰でもスタックオーバーフローしません。

---

## イテレータによる再帰の置き換え

Rust では多くの再帰をイテレータで書き直せます。状態機械として表現することで、シンプルかつ効率的になります。

```rust
struct FibIter {
    a: u64,
    b: u64,
}

impl FibIter {
    fn new() -> Self {
        FibIter { a: 0, b: 1 }
    }
}

impl Iterator for FibIter {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.a;
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(result)
    }
}

// 使用例
let first_10: Vec<u64> = FibIter::new().take(10).collect();
// → [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

---

## 相互再帰

2つ以上の関数が互いに呼び合う場合を**相互再帰**と言います。

```rust
fn is_even(n: u32) -> bool {
    if n == 0 { true } else { is_odd(n - 1) }
}

fn is_odd(n: u32) -> bool {
    if n == 0 { false } else { is_even(n - 1) }
}
```

---

## 木構造の再帰処理

木構造は再帰と相性が良いデータ構造です。

```rust
enum Tree<T> {
    Leaf,
    Node(T, Box<Tree<T>>, Box<Tree<T>>),
}

impl<T> Tree<T> {
    fn depth(&self) -> usize {
        match self {
            Tree::Leaf => 0,
            Tree::Node(_, left, right) => 1 + left.depth().max(right.depth()),
        }
    }

    fn count(&self) -> usize {
        match self {
            Tree::Leaf => 0,
            Tree::Node(_, left, right) => 1 + left.count() + right.count(),
        }
    }
}
```

---

## まとめ

| 手法 | スタック消費 | 可読性 | 用途 |
|------|------------|--------|------|
| 素朴な再帰 | 多い（深さに比例） | 高い | 浅い再帰、木構造 |
| 末尾再帰スタイル | 少ない（TCO期待） | 中程度 | 線形再帰 |
| Trampoline | 一定（ヒープ使用） | 低い | 深い再帰、TCO保証が必要な場合 |
| イテレータ | 一定 | 高い | シーケンス生成 |

Rust では**イテレータ**を優先し、木構造などには**素朴な再帰**、深さが問題になる場合は**Trampoline**を使うのが実践的です。
