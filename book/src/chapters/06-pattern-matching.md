# パターンマッチング（Pattern Matching）

## 概要

**パターンマッチング**は関数型プログラミングの重要な機能です。Rustの `match` 式は強力で、すべてのパターンを網羅することが保証されます（網羅性チェック）。

## 基本的な `match`

```rust
let number = 7;

match number {
    1 => println!("一"),
    2 | 3 => println!("二か三"),
    4..=6 => println!("四から六"),
    n if n > 6 => println!("七以上: {}", n),
    _ => println!("その他"),
}
```

## 列挙型（enum）のパターンマッチング

```rust
#[derive(Debug)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

impl Shape {
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }
}
```

## 再帰的な列挙型（式の評価）

```rust
#[derive(Debug)]
pub enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

impl Expr {
    pub fn eval(&self) -> f64 {
        match self {
            Expr::Num(n) => *n,
            Expr::Add(a, b) => a.eval() + b.eval(),
            Expr::Mul(a, b) => a.eval() * b.eval(),
            Expr::Neg(e) => -e.eval(),
        }
    }
}

// (2 + 3) * -(4) = -20
let expr = Expr::Mul(
    Box::new(Expr::Add(
        Box::new(Expr::Num(2.0)),
        Box::new(Expr::Num(3.0)),
    )),
    Box::new(Expr::Neg(Box::new(Expr::Num(4.0)))),
);
println!("{}", expr.eval()); // -20
```

## タプルのマッチング

```rust
fn classify_point(x: i32, y: i32) -> &'static str {
    match (x, y) {
        (0, 0) => "原点",
        (x, 0) if x > 0 => "正のX軸上",
        (0, y) if y > 0 => "正のY軸上",
        (x, y) if x > 0 && y > 0 => "第1象限",
        _ => "その他",
    }
}
```

## `if let` と `while let`

特定のパターンだけを扱う場合、`if let` が便利です：

```rust
let value: Option<i32> = Some(42);

// match より簡潔
if let Some(n) = value {
    println!("値は {}", n);
}

// while let でイテレーション
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("{}", top);
}
```

## 構造体の分割代入（Destructuring）

```rust
struct Point { x: i32, y: i32 }
let p = Point { x: 3, y: 7 };

let Point { x, y } = p;
println!("x={}, y={}", x, y);
```

## 演習

`crates/pattern_matching/src/main.rs` を参照してください。

```bash
cargo run -p pattern_matching
cargo test -p pattern_matching
```
