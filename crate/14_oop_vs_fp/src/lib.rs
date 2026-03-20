// 第14章: OOP vs 関数型プログラミング比較

// ─── 1. Strategy パターン: OOP vs FP ─────────────────────────

// OOP スタイル: トレイトオブジェクト
pub trait SortStrategy {
    fn sort(&self, data: &mut Vec<i32>);
}

pub struct BubbleSort;
impl SortStrategy for BubbleSort {
    fn sort(&self, data: &mut Vec<i32>) {
        let n = data.len();
        for i in 0..n {
            for j in 0..n - 1 - i {
                if data[j] > data[j + 1] {
                    data.swap(j, j + 1);
                }
            }
        }
    }
}

pub struct Sorter {
    strategy: Box<dyn SortStrategy>,
}

impl Sorter {
    pub fn new(strategy: Box<dyn SortStrategy>) -> Self {
        Sorter { strategy }
    }
    pub fn sort(&self, data: &mut Vec<i32>) {
        self.strategy.sort(data);
    }
}

// FP スタイル: 関数を引数として渡す
pub fn sort_with<F>(data: &mut Vec<i32>, strategy: F)
where
    F: Fn(&mut Vec<i32>),
{
    strategy(data);
}

pub fn bubble_sort_fn(data: &mut Vec<i32>) {
    let n = data.len();
    for i in 0..n {
        for j in 0..n - 1 - i {
            if data[j] > data[j + 1] {
                data.swap(j, j + 1);
            }
        }
    }
}

// ─── 2. Decorator パターン: OOP vs FP ────────────────────────

// OOP スタイル
pub trait Logger {
    fn log(&self, message: &str) -> String;
}

pub struct PlainLogger;
impl Logger for PlainLogger {
    fn log(&self, message: &str) -> String {
        message.to_string()
    }
}

pub struct TimestampLogger {
    inner: Box<dyn Logger>,
    timestamp: String,
}

impl TimestampLogger {
    pub fn new(inner: Box<dyn Logger>, timestamp: &str) -> Self {
        TimestampLogger { inner, timestamp: timestamp.to_string() }
    }
}

impl Logger for TimestampLogger {
    fn log(&self, message: &str) -> String {
        format!("[{}] {}", self.timestamp, self.inner.log(message))
    }
}

// FP スタイル: 関数合成でデコレート
pub fn with_timestamp<'a>(f: impl Fn(&str) -> String + 'a, ts: &'a str) -> impl Fn(&str) -> String + 'a {
    move |msg| format!("[{}] {}", ts, f(msg))
}

pub fn with_prefix<'a>(f: impl Fn(&str) -> String + 'a, prefix: &'a str) -> impl Fn(&str) -> String + 'a {
    move |msg| format!("{} {}", prefix, f(msg))
}

// ─── 3. 状態管理: 可変オブジェクト vs 不変データ変換 ─────────

// OOP スタイル: 可変状態
pub struct Counter {
    count: i32,
    step: i32,
}

impl Counter {
    pub fn new(step: i32) -> Self {
        Counter { count: 0, step }
    }
    pub fn increment(&mut self) {
        self.count += self.step;
    }
    pub fn get(&self) -> i32 {
        self.count
    }
}

// FP スタイル: 不変データ変換
#[derive(Debug, Clone, PartialEq)]
pub struct CounterState {
    pub count: i32,
    pub step: i32,
}

pub fn counter_new(step: i32) -> CounterState {
    CounterState { count: 0, step }
}

pub fn counter_increment(state: &CounterState) -> CounterState {
    CounterState {
        count: state.count + state.step,
        ..*state
    }
}

// ─── 4. Observer パターン: OOP vs FP ─────────────────────────

// OOP スタイル
pub struct EventSource {
    listeners: Vec<Box<dyn Fn(i32)>>,
}

impl EventSource {
    pub fn new() -> Self {
        EventSource { listeners: Vec::new() }
    }
    pub fn subscribe(&mut self, listener: impl Fn(i32) + 'static) {
        self.listeners.push(Box::new(listener));
    }
    pub fn emit(&self, value: i32) {
        for listener in &self.listeners {
            listener(value);
        }
    }
}

impl Default for EventSource {
    fn default() -> Self {
        Self::new()
    }
}

// FP スタイル: イベントリストをパイプラインで処理
pub fn process_events<F>(events: &[i32], handler: F) -> Vec<String>
where
    F: Fn(i32) -> String,
{
    events.iter().map(|&e| handler(e)).collect()
}

// ─── 5. 継承 vs 合成 ─────────────────────────────────────────

// FP スタイルの合成: 関数を組み合わせる
pub fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

pub fn double(x: i32) -> i32 { x * 2 }
pub fn add_one(x: i32) -> i32 { x + 1 }
pub fn square(x: i32) -> i32 { x * x }

// ─── 6. 同じ問題を両スタイルで: 割引計算 ────────────────────

// OOP スタイル
pub trait DiscountPolicy {
    fn apply(&self, price: f64) -> f64;
}

pub struct PercentDiscount(f64);
impl DiscountPolicy for PercentDiscount {
    fn apply(&self, price: f64) -> f64 {
        price * (1.0 - self.0 / 100.0)
    }
}

pub struct FixedDiscount(f64);
impl DiscountPolicy for FixedDiscount {
    fn apply(&self, price: f64) -> f64 {
        (price - self.0).max(0.0)
    }
}

// FP スタイル: 関数として表現
pub type DiscountFn = fn(f64) -> f64;

pub fn percent_discount(rate: f64) -> impl Fn(f64) -> f64 {
    move |price| price * (1.0 - rate / 100.0)
}

pub fn fixed_discount(amount: f64) -> impl Fn(f64) -> f64 {
    move |price| (price - amount).max(0.0)
}

pub fn apply_discounts(price: f64, discounts: &[&dyn Fn(f64) -> f64]) -> f64 {
    discounts.iter().fold(price, |p, f| f(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Strategy パターン
    #[test]
    fn test_oop_strategy() {
        let sorter = Sorter::new(Box::new(BubbleSort));
        let mut data = vec![3, 1, 4, 1, 5];
        sorter.sort(&mut data);
        assert_eq!(data, vec![1, 1, 3, 4, 5]);
    }

    #[test]
    fn test_fp_strategy() {
        let mut data = vec![3, 1, 4, 1, 5];
        sort_with(&mut data, bubble_sort_fn);
        assert_eq!(data, vec![1, 1, 3, 4, 5]);
    }

    #[test]
    fn test_fp_strategy_with_closure() {
        let mut data = vec![3, 1, 4, 1, 5];
        // クロージャを直接渡せる
        sort_with(&mut data, |d| d.sort());
        assert_eq!(data, vec![1, 1, 3, 4, 5]);
    }

    // Decorator パターン
    #[test]
    fn test_oop_decorator() {
        let logger = TimestampLogger::new(Box::new(PlainLogger), "2024-01-01");
        assert_eq!(logger.log("hello"), "[2024-01-01] hello");
    }

    #[test]
    fn test_fp_decorator() {
        let plain = |msg: &str| msg.to_string();
        let with_ts = with_timestamp(plain, "2024-01-01");
        assert_eq!(with_ts("hello"), "[2024-01-01] hello");
    }

    #[test]
    fn test_fp_decorator_composed() {
        let plain = |msg: &str| msg.to_string();
        let with_ts = with_timestamp(plain, "2024-01-01");
        let with_pfx = with_prefix(with_ts, "[INFO]");
        assert_eq!(with_pfx("hello"), "[INFO] [2024-01-01] hello");
    }

    // 状態管理
    #[test]
    fn test_oop_mutable_state() {
        let mut counter = Counter::new(2);
        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 4);
    }

    #[test]
    fn test_fp_immutable_state() {
        let s0 = counter_new(2);
        let s1 = counter_increment(&s0);
        let s2 = counter_increment(&s1);

        assert_eq!(s0.count, 0); // 元の状態は変わらない
        assert_eq!(s2.count, 4);
    }

    // Observer パターン
    #[test]
    fn test_oop_observer() {
        let mut source = EventSource::new();
        let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::<i32>::new()));
        let results_clone = results.clone();
        source.subscribe(move |v| {
            results_clone.lock().unwrap().push(v * 2);
        });
        source.emit(10);
        source.emit(20);
        let data = results.lock().unwrap();
        assert_eq!(*data, vec![20, 40]);
    }

    #[test]
    fn test_fp_event_pipeline() {
        let events = vec![1, 2, 3, 4, 5];
        let result = process_events(&events, |e| format!("event:{}", e * 10));
        assert_eq!(result[0], "event:10");
        assert_eq!(result[4], "event:50");
    }

    // 合成
    #[test]
    fn test_fp_composition() {
        let double_then_add_one = compose(double, add_one);
        assert_eq!(double_then_add_one(5), 11); // 5*2+1

        let pipeline = compose(compose(double, add_one), square);
        assert_eq!(pipeline(3), 49); // (3*2+1)^2 = 7^2 = 49
    }

    // 割引計算
    #[test]
    fn test_oop_discount() {
        let policy = PercentDiscount(10.0);
        assert_eq!(policy.apply(1000.0), 900.0);

        let policy = FixedDiscount(200.0);
        assert_eq!(policy.apply(1000.0), 800.0);
    }

    #[test]
    fn test_fp_discount() {
        let ten_percent = percent_discount(10.0);
        assert_eq!(ten_percent(1000.0), 900.0);

        let two_hundred_off = fixed_discount(200.0);
        assert_eq!(two_hundred_off(1000.0), 800.0);
    }

    #[test]
    fn test_fp_discount_composition() {
        let ten_percent = percent_discount(10.0);
        let two_hundred_off = fixed_discount(200.0);

        // 複数の割引を連続適用
        let discounts: Vec<&dyn Fn(f64) -> f64> = vec![&ten_percent, &two_hundred_off];
        let final_price = apply_discounts(1000.0, &discounts);
        assert_eq!(final_price, 700.0); // 1000 * 0.9 - 200 = 700
    }
}
