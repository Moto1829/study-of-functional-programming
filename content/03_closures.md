# 第3章: クロージャと高階関数

## 目次

1. [クロージャの基本構文](#1-クロージャの基本構文)
2. [環境のキャプチャ](#2-環境のキャプチャ)
3. [Fn / FnMut / FnOnce トレイト](#3-fn--fnmut--fnonce-トレイト)
4. [高階関数](#4-高階関数)
5. [関数ポインタとクロージャの違い](#5-関数ポインタとクロージャの違い)
6. [カリー化の模倣パターン](#6-カリー化の模倣パターン)
7. [章末演習問題](#7-章末演習問題)

---

## 1. クロージャの基本構文

クロージャとは、変数に束縛したり関数に渡したりできる**匿名関数**です。Rustでは `|引数| 式` という構文で書きます。

```rust
// 最もシンプルなクロージャ
let add_one = |x| x + 1;
println!("{}", add_one(5)); // 6

// 型を明示することもできる
let add_one: impl Fn(i32) -> i32 = |x: i32| -> i32 { x + 1 };

// 複数行にまたがるクロージャ（ブロック式を使う）
let greet = |name: &str| {
    let message = format!("Hello, {}!", name);
    message
};
println!("{}", greet("Alice")); // Hello, Alice!
```

通常の関数と比べて、クロージャには次の特徴があります。

| 特徴 | 通常の関数 (`fn`) | クロージャ |
|------|-----------------|----------|
| 引数の型注釈 | 必須 | 省略可能（推論される） |
| 周囲の変数へのアクセス | 不可 | 可能（キャプチャ） |
| 値として渡せる | 関数ポインタとして渡せる | トレイトオブジェクトとして渡せる |

---

## 2. 環境のキャプチャ

クロージャが定義されたスコープの変数を参照・使用することを**キャプチャ**と呼びます。Rustはキャプチャの方法を3つ持ちます。

### 2.1 参照によるキャプチャ（デフォルト）

```rust
fn main() {
    let message = String::from("Hello");

    // message を不変参照でキャプチャする
    let print_message = || println!("{}", message);

    print_message(); // Hello
    print_message(); // Hello
    println!("{}", message); // クロージャの外でも使える
}
```

### 2.2 可変参照によるキャプチャ

```rust
fn main() {
    let mut count = 0;

    // count を可変参照でキャプチャする
    let mut increment = || {
        count += 1;
        println!("count = {}", count);
    };

    increment(); // count = 1
    increment(); // count = 2
    // ここで increment を使い終わったので count が返る
    println!("最終: {}", count); // 最終: 2
}
```

### 2.3 move クロージャ（所有権ごとキャプチャ）

`move` キーワードを付けると、クロージャは参照ではなく**値の所有権**を奪います。スレッドにデータを渡すときや、クロージャをスコープ外で使うときに必要です。

```rust
use std::thread;

fn main() {
    let data = vec![1, 2, 3];

    // `data` の所有権をクロージャに移動する
    let handle = thread::spawn(move || {
        println!("スレッド内: {:?}", data);
    });

    // data はもう使えない（所有権がクロージャに移った）
    // println!("{:?}", data); // コンパイルエラー!

    handle.join().unwrap();
}
```

`move` が必要な理由: スレッドはいつまで生きるかわからないため、参照が dangling になる可能性があります。`move` によって所有権ごと渡すことで安全性を保証します。

---

## 3. Fn / FnMut / FnOnce トレイト

Rustのクロージャは、キャプチャの方法によって自動的に3つのトレイトのうちいずれか（複数の場合もある）を実装します。

### トレイト一覧

| トレイト | 呼び出し方 | 実装条件 |
|---------|-----------|---------|
| `FnOnce` | 1回のみ | すべてのクロージャが実装する（所有権を消費するものを含む） |
| `FnMut` | 複数回（可変） | キャプチャした値を変更するクロージャ |
| `Fn` | 複数回（不変） | キャプチャした値を変更しないクロージャ |

これらは継承関係にあります: `Fn: FnMut: FnOnce`

つまり `Fn` を実装していれば `FnMut` と `FnOnce` も実装していますが、逆は成り立ちません。

### FnOnce の例

```rust
fn consume_once<F: FnOnce() -> String>(f: F) {
    let result = f(); // 1回だけ呼べる
    println!("{}", result);
    // f(); // コンパイルエラー! FnOnce は2回呼べない
}

fn main() {
    let name = String::from("Alice");

    // name の所有権を消費するクロージャ → FnOnce のみ実装
    let greeting = move || format!("Hello, {}!", name);

    consume_once(greeting);
}
```

### FnMut の例

```rust
fn apply_mutating<F: FnMut(i32) -> i32>(mut f: F, values: &[i32]) -> Vec<i32> {
    values.iter().map(|&x| f(x)).collect()
}

fn main() {
    let mut multiplier = 1;

    // 呼び出すたびに multiplier が増える → FnMut
    let mut scaling = |x: i32| {
        multiplier += 1;
        x * multiplier
    };

    let result = apply_mutating(&mut scaling, &[1, 2, 3]);
    println!("{:?}", result); // [2, 6, 12] （multiplierが2,3,4と増える）
}
```

### Fn の例

```rust
fn apply_to_all<F: Fn(i32) -> i32>(f: F, values: &[i32]) -> Vec<i32> {
    values.iter().map(|&x| f(x)).collect()
}

fn main() {
    let offset = 10;

    // offset を不変参照でキャプチャ → Fn を実装
    let add_offset = |x| x + offset;

    let result = apply_to_all(add_offset, &[1, 2, 3]);
    println!("{:?}", result); // [11, 12, 13]
}
```

### どのトレイトを使うべきか

関数の引数に制約を書くとき、できるだけ緩やかなトレイトを選ぶと呼び出し元の柔軟性が増します。

```rust
// 最も厳しい制約（FnOnce を要求）→ 最も多くのクロージャを受け付ける
fn run_once<F: FnOnce()>(f: F) { f(); }

// 中程度の制約（FnMut を要求）→ 状態を持つクロージャも受け付ける
fn run_times<F: FnMut()>(mut f: F, n: usize) {
    for _ in 0..n { f(); }
}

// 最も厳しい制約（Fn を要求）→ 純粋なクロージャのみ受け付ける
fn run_parallel<F: Fn() + Send + Sync>(f: F) {
    f();
}
```

---

## 4. 高階関数

**高階関数**とは、関数を引数に取る、または関数を戻り値として返す関数のことです。

### 4.1 関数を引数に取る

```rust
fn apply<T, U, F: Fn(T) -> U>(f: F, value: T) -> U {
    f(value)
}

fn main() {
    let double = |x: i32| x * 2;
    let to_upper = |s: &str| s.to_uppercase();

    println!("{}", apply(double, 5));         // 10
    println!("{}", apply(to_upper, "hello")); // HELLO
}
```

### 4.2 関数を戻り値にする

Rustでクロージャを返すには `impl Fn(...)` 構文を使います。

```rust
/// 加算器を生成して返す
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

fn main() {
    let add5 = make_adder(5);
    let add10 = make_adder(10);

    println!("{}", add5(3));  // 8
    println!("{}", add10(3)); // 13
}
```

トレイトオブジェクト (`Box<dyn Fn>`) を使うと、動的ディスパッチが必要な場面でも関数を返せます。

```rust
/// 条件によって異なる変換関数を返す
fn choose_transform(invert: bool) -> Box<dyn Fn(i32) -> i32> {
    if invert {
        Box::new(|x| -x)
    } else {
        Box::new(|x| x)
    }
}

fn main() {
    let t = choose_transform(true);
    println!("{}", t(42)); // -42
}
```

### 4.3 apply_twice パターン

```rust
/// 関数を同じ引数に2回適用する汎用関数
fn apply_twice<T: Clone, F: Fn(T) -> T>(f: F, x: T) -> T {
    f(f(x.clone()))
}

fn main() {
    let double = |x: i32| x * 2;
    println!("{}", apply_twice(double, 3)); // 12 (3 → 6 → 12)

    let exclaim = |s: String| format!("{}!", s);
    println!("{}", apply_twice(exclaim, String::from("Hi"))); // Hi!!
}
```

---

## 5. 関数ポインタとクロージャの違い

**関数ポインタ** (`fn`) は環境をキャプチャしない関数への参照です。`fn` 型はサイズが固定されておりコストが低い一方、クロージャのようにデータを保持することはできません。

```rust
// 関数ポインタを受け取る関数
fn apply_fn_ptr(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn double(x: i32) -> i32 { x * 2 }

fn main() {
    // 通常の関数は fn 型として渡せる
    println!("{}", apply_fn_ptr(double, 5)); // 10

    // 環境をキャプチャしないクロージャも fn 型として渡せる
    println!("{}", apply_fn_ptr(|x| x + 1, 5)); // 6

    let offset = 3;
    // 環境をキャプチャするクロージャは fn 型では渡せない
    // apply_fn_ptr(|x| x + offset, 5); // コンパイルエラー!
    //                    ^^^^^^ captures environment
}
```

### 使い分けの指針

| 用途 | 型 |
|------|-----|
| 環境をキャプチャしない・FFIで使う | `fn(T) -> U` |
| 環境をキャプチャする・一般的な高階関数 | `impl Fn(T) -> U` |
| 型消去が必要・ヒープ確保を許容する | `Box<dyn Fn(T) -> U>` |

---

## 6. カリー化の模倣パターン

**カリー化 (Currying)** とは、複数の引数を取る関数を、1つの引数を取る関数の連鎖に変換することです。Haskellなどの言語では組み込みですが、Rustでは`クロージャを返す関数`によって模倣できます。

### 基本パターン

```rust
/// カリー化された加算: add(x) は |y| x + y を返す
fn add(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

fn main() {
    let add5 = add(5);

    println!("{}", add5(3));  // 8
    println!("{}", add5(10)); // 15

    // 即時適用
    println!("{}", add(3)(4)); // 7
}
```

### 3引数のカリー化

```rust
/// 3引数をカリー化した形で積算する
/// 安定版 Rust では `impl Fn -> impl Fn` が使えないため `Box<dyn Fn>` で返す
fn multiply3(x: i32) -> impl Fn(i32) -> Box<dyn Fn(i32) -> i32> {
    move |y| Box::new(move |z| x * y * z)
}

fn main() {
    let step1 = multiply3(2);
    let step2 = step1(3);
    println!("{}", step2(4)); // 24

    // 連鎖して書くこともできる
    println!("{}", multiply3(2)(3)(4)); // 24
}
```

### カリー化と部分適用の活用例

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    // add(10) で「10を足す関数」を作り、map に渡す
    let result: Vec<i32> = numbers.iter().map(|&x| add(10)(x)).collect();
    println!("{:?}", result); // [11, 12, 13, 14, 15]
}
```

---

## 7. 章末演習問題

### 問題 1: compose 関数の実装

2つの関数 `f: B -> C` と `g: A -> B` を受け取り、`A -> C` となる合成関数 `f(g(x))` を返す `compose` 関数を実装してください。

```rust
fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(B) -> C,
    G: Fn(A) -> B,
{
    // ここを実装してください
    todo!()
}

fn main() {
    let double = |x: i32| x * 2;
    let add_one = |x: i32| x + 1;

    let double_then_add = compose(add_one, double);
    println!("{}", double_then_add(5)); // 11 (5*2 + 1)
}
```

<details>
<summary>ヒント</summary>

`move` クロージャを使って `f` と `g` を所有権ごとキャプチャし、`f(g(x))` を計算するクロージャを返してください。

</details>

---

### 問題 2: memoize 関数の実装

引数 `i32` に対する計算結果をキャッシュするメモ化関数を実装してください。同じ引数で2回目に呼ばれたときは計算をスキップして、キャッシュの値を返すようにしてください。

```rust
use std::collections::HashMap;

struct Memoized<F> {
    func: F,
    cache: HashMap<i32, i32>,
}

impl<F: Fn(i32) -> i32> Memoized<F> {
    fn new(func: F) -> Self {
        // ここを実装してください
        todo!()
    }

    fn call(&mut self, x: i32) -> i32 {
        // ここを実装してください
        todo!()
    }
}
```

<details>
<summary>ヒント</summary>

`HashMap::entry().or_insert_with()` を使うとエレガントに書けます。ただし `self.func` への参照と `self.cache` への可変参照を同時に保持できないため、`entry` と `insert` を分けて書く必要があります。

</details>

---

### 問題 3: パイプライン演算子の模倣

Rustには `|>` のようなパイプライン演算子はありませんが、メソッドチェーンで模倣できます。以下のように `Pipeline` 構造体を定義して、`.pipe(f)` メソッドで変換を連鎖できるようにしてください。

```rust
struct Pipeline<T>(T);

impl<T> Pipeline<T> {
    fn new(value: T) -> Self {
        Pipeline(value)
    }

    fn pipe<U, F: Fn(T) -> U>(self, f: F) -> Pipeline<U> {
        // ここを実装してください
        todo!()
    }

    fn value(self) -> T {
        self.0
    }
}

fn main() {
    let result = Pipeline::new(5)
        .pipe(|x| x * 2)    // 10
        .pipe(|x| x + 1)    // 11
        .pipe(|x| x * x)    // 121
        .value();

    println!("{}", result); // 121
}
```

<details>
<summary>ヒント</summary>

`pipe` の中で `f(self.0)` を計算し、`Pipeline` に包んで返してください。`self` を消費するので所有権が正しく移動します。

</details>

---

## まとめ

この章では以下を学びました。

- **クロージャ**は匿名関数であり、周囲の変数をキャプチャできる
- キャプチャの方法（不変参照・可変参照・move）によって `Fn` / `FnMut` / `FnOnce` が決まる
- **高階関数**は関数を引数や戻り値として扱うことで、汎用的で合成可能なコードを書ける
- **関数ポインタ** (`fn`) は環境をキャプチャできないが軽量である
- **カリー化**は「クロージャを返す関数」で模倣でき、部分適用を実現する手段になる

次章では `Iterator` トレイトと遅延評価を学び、クロージャをさらに活用した宣言的なデータ処理を扱います。
