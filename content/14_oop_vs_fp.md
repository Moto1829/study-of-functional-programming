# 第14章: OOP vs 関数型プログラミング比較

## はじめに

オブジェクト指向プログラミング（OOP）と関数型プログラミング（FP）は、問題への異なるアプローチを提供します。Rust は両方のスタイルをサポートするため、比較しながら理解を深めましょう。

---

## 設計思想の違い

| 観点 | OOP | 関数型 |
|------|-----|--------|
| 基本単位 | オブジェクト（状態 + 振る舞い） | 関数（入力 → 出力） |
| 状態 | 可変（メソッドで変更） | 不変（新しい値を生成） |
| コード再利用 | 継承・ポリモーフィズム | 関数合成・高階関数 |
| 副作用 | 暗黙的（オブジェクト内部） | 明示的・局所化 |

---

## Strategy パターン: OOP vs FP

### OOP スタイル（トレイトオブジェクト）

```rust
pub trait SortStrategy {
    fn sort(&self, data: &mut Vec<i32>);
}

pub struct Sorter {
    strategy: Box<dyn SortStrategy>,
}

impl Sorter {
    pub fn sort(&self, data: &mut Vec<i32>) {
        self.strategy.sort(data);
    }
}

// 使用
let sorter = Sorter::new(Box::new(BubbleSort));
sorter.sort(&mut data);
```

### FP スタイル（関数を引数として渡す）

```rust
pub fn sort_with<F: Fn(&mut Vec<i32>)>(data: &mut Vec<i32>, strategy: F) {
    strategy(data);
}

// クロージャを直接渡せる
sort_with(&mut data, |d| d.sort());
sort_with(&mut data, bubble_sort_fn);
```

**FP の利点:** ボイラープレートが少なく、クロージャを直接渡せる。

---

## Decorator パターン: OOP vs FP

### OOP スタイル（ラッパークラス）

```rust
pub struct TimestampLogger {
    inner: Box<dyn Logger>,
    timestamp: String,
}

impl Logger for TimestampLogger {
    fn log(&self, message: &str) -> String {
        format!("[{}] {}", self.timestamp, self.inner.log(message))
    }
}
```

### FP スタイル（関数合成）

```rust
pub fn with_timestamp<'a>(
    f: impl Fn(&str) -> String + 'a,
    ts: &'a str,
) -> impl Fn(&str) -> String + 'a {
    move |msg| format!("[{}] {}", ts, f(msg))
}

// 複数のデコレータを合成
let plain = |msg: &str| msg.to_string();
let with_ts = with_timestamp(plain, "2024-01-01");
let with_pfx = with_prefix(with_ts, "[INFO]");
// "[INFO] [2024-01-01] hello"
```

**FP の利点:** 新しいクラスを作らずに、関数を組み合わせてデコレートできる。

---

## 状態管理: 可変オブジェクト vs 不変データ変換

### OOP スタイル（可変状態）

```rust
pub struct Counter {
    count: i32,
    step: i32,
}

impl Counter {
    pub fn increment(&mut self) {
        self.count += self.step;
    }
}

let mut counter = Counter::new(2);
counter.increment();
counter.increment();
// counter.count == 4（状態が変化した）
```

### FP スタイル（不変データ変換）

```rust
#[derive(Clone)]
pub struct CounterState { count: i32, step: i32 }

pub fn counter_increment(state: &CounterState) -> CounterState {
    CounterState { count: state.count + state.step, ..*state }
}

let s0 = counter_new(2);
let s1 = counter_increment(&s0);
let s2 = counter_increment(&s1);

assert_eq!(s0.count, 0); // 元の状態は変わらない！
assert_eq!(s2.count, 4);
```

**FP の利点:** 過去の状態が保持される。テストが容易で、バグが追跡しやすい。

---

## Observer パターン: OOP vs FP

### OOP スタイル（リスナー登録）

```rust
let mut source = EventSource::new();
source.subscribe(|v| println!("received: {}", v));
source.emit(42);
```

### FP スタイル（イベントをデータとして扱う）

```rust
let events = vec![1, 2, 3, 4, 5];
let results = process_events(&events, |e| format!("event:{}", e * 10));
// イテレータのパイプラインとして処理
```

---

## 継承 vs 合成

OOP では継承でコードを再利用しますが、FP では**関数合成**を使います。

```rust
pub fn compose<A, B, C>(
    f: impl Fn(A) -> B,
    g: impl Fn(B) -> C,
) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

// 関数を組み合わせてパイプラインを作る
let pipeline = compose(compose(double, add_one), square);
pipeline(3) // (3*2+1)^2 = 49
```

---

## 同じ問題を両スタイルで: 割引計算

### OOP スタイル

```rust
pub trait DiscountPolicy {
    fn apply(&self, price: f64) -> f64;
}

let policy = PercentDiscount(10.0);
let price = policy.apply(1000.0); // 900.0
```

### FP スタイル

```rust
let ten_percent = percent_discount(10.0);   // fn(f64) -> f64 を返す
let two_hundred_off = fixed_discount(200.0);

// 複数の割引を連続適用
let discounts: Vec<&dyn Fn(f64) -> f64> = vec![&ten_percent, &two_hundred_off];
let final_price = apply_discounts(1000.0, &discounts);
// 1000 * 0.9 - 200 = 700.0
```

**FP の利点:** 割引の組み合わせが柔軟で、新しいクラスを追加せずに拡張できる。

---

## まとめ: どちらを使うか

| 場面 | 推奨スタイル |
|------|------------|
| 複雑な状態を持つエンティティ | OOP（構造体 + メソッド） |
| データ変換パイプライン | FP（イテレータ・関数合成） |
| エラー処理 | FP（`Result`・`Option`） |
| プラグイン・策略の切り替え | どちらも可（OOP: トレイト、FP: 関数） |
| 並行処理 | FP（不変データで安全） |

Rust では OOP と FP を**混在させる**ことが普通です。トレイトでポリモーフィズムを実現しつつ、イテレータと関数合成でデータを処理するスタイルが Rust らしい書き方です。
