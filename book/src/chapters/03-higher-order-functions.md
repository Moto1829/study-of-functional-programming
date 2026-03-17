# 高階関数（Higher-Order Functions）

## 概要

**高階関数**とは、以下のいずれかまたは両方の性質を持つ関数です：

1. **関数を引数として受け取る**
2. **関数を戻り値として返す**

これにより、コードの再利用性と抽象化が大幅に向上します。

## 代表的な高階関数

### `map` — 変換

各要素に関数を適用して新しいリストを生成します：

```rust
let numbers = vec![1, 2, 3, 4, 5];
let squared: Vec<i32> = numbers.iter().map(|&n| n * n).collect();
// [1, 4, 9, 16, 25]
```

### `filter` — 絞り込み

条件を満たす要素だけを残します：

```rust
let evens: Vec<i32> = numbers.iter().filter(|&&n| n % 2 == 0).cloned().collect();
// [2, 4]
```

### `fold` / `reduce` — 畳み込み

リストを1つの値に集約します：

```rust
let sum: i32 = numbers.iter().fold(0, |acc, &n| acc + n);
// 15
```

## 関数を返す関数

```rust
// n を加算する関数を生成して返す
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

let add5 = make_adder(5);
println!("{}", add5(10)); // 15
println!("{}", add5(20)); // 25
```

## 関数の合成（Function Composition）

```rust
fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

let add3 = make_adder(3);
let double = |x: i32| x * 2;
let add3_then_double = compose(add3, double);

println!("{}", add3_then_double(4)); // (4+3)*2 = 14
```

## 実践例: データ処理パイプライン

```rust
let scores = vec![85, 42, 93, 67, 78, 55, 91];

// 70点以上の点数を取り出し、10点加算してソート
let processed: Vec<i32> = scores
    .iter()
    .filter(|&&s| s >= 70)
    .map(|&s| s + 10)
    .collect();
```

## 演習

`crates/higher_order_functions/src/main.rs` を参照してください。

```bash
cargo run -p higher_order_functions
cargo test -p higher_order_functions
```
