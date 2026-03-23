# 第15章: Lenses / Optics

## はじめに

関数型プログラミングでは不変データ構造を使うのが基本です。しかし「ネストした構造の一部だけを更新したい」場合、毎回全フィールドを書き直すのは面倒です。

**Lens（レンズ）** はこの問題を解決する抽象です。「データ構造の中の特定の値へのアクセス（getter）と更新（setter）」を一つの値として表現します。

---

## Lens の基本概念

Lens は次の2つの操作をまとめたものです：

- **get**: 構造体 `S` から値 `A` を取り出す
- **set**: 構造体 `S` の中の値 `A` を新しい値に置き換えた新しい `S` を返す

```rust
struct Lens<S, A> {
    get: fn(&S) -> A,
    set: fn(S, A) -> S,
}
```

`set` は元の `S` を変更せず、新しい `S` を返します。これが関数型の「不変更新」です。

---

## 手作業による Lens

まず Lens を使わない素朴な実装で問題を確認します。

```rust
#[derive(Debug, Clone)]
struct Address {
    city: String,
    zip: String,
}

#[derive(Debug, Clone)]
struct Person {
    name: String,
    age: u32,
    address: Address,
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            city: "Tokyo".to_string(),
            zip: "100-0001".to_string(),
        },
    };

    // city だけ更新したい → 全フィールドを書き直す必要がある
    let updated = Person {
        address: Address {
            city: "Osaka".to_string(),
            ..person.address.clone()
        },
        ..person.clone()
    };

    println!("{:?}", updated);
}
```

ネストが深くなるほどこのコードは冗長になります。Lens はこれを解決します。

---

## Rust で Lens を実装する

```rust
#[derive(Debug, Clone, PartialEq)]
struct Address {
    city: String,
    zip: String,
}

#[derive(Debug, Clone, PartialEq)]
struct Person {
    name: String,
    age: u32,
    address: Address,
}

// Lens の定義: get は参照ではなく所有値を返す（A: Clone が必要）
struct Lens<S, A> {
    get: fn(&S) -> A,
    set: fn(S, A) -> S,
}

impl<S: Clone, A: Clone> Lens<S, A> {
    fn view(&self, s: &S) -> A {
        (self.get)(s)
    }

    fn update(&self, s: S, a: A) -> S {
        (self.set)(s, a)
    }

    fn modify<F>(&self, s: S, f: F) -> S
    where
        F: FnOnce(A) -> A,
    {
        let a = (self.get)(&s);
        self.update(s, f(a))
    }
}

// Person → address の Lens
fn person_address_lens() -> Lens<Person, Address> {
    Lens {
        get: |p| p.address.clone(),
        set: |p, addr| Person { address: addr, ..p },
    }
}

// Address → city の Lens
fn address_city_lens() -> Lens<Address, String> {
    Lens {
        get: |a| a.city.clone(),
        set: |a, city| Address { city, ..a },
    }
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            city: "Tokyo".to_string(),
            zip: "100-0001".to_string(),
        },
    };

    let addr_lens = person_address_lens();
    let city_lens = address_city_lens();

    // address.city を取得
    let address = addr_lens.view(&person);
    let city = city_lens.view(&address);
    println!("City: {}", city); // "Tokyo"

    // address.city を "Osaka" に更新
    let new_address = city_lens.update(addr_lens.view(&person), "Osaka".to_string());
    let updated_person = addr_lens.update(person.clone(), new_address);
    println!("{:?}", updated_person);
}
```

---

## Lens の合成

Lens の最大の強みは**合成**です。`Person → Address` の Lens と `Address → city` の Lens を合成すると `Person → city` の Lens が作れます。

```rust
// 2つの Lens を合成する関数 (S → A の Lens と A → B の Lens から S → B の getter を返す)
pub fn compose<'a, S: Clone, A: Clone, B: Clone>(
    outer: &'a Lens<S, A>,
    inner: &'a Lens<A, B>,
) -> impl Fn(&S) -> B + 'a {
    move |s| inner.view(&outer.view(s))
}

// 合成した更新関数
fn set_city(person: Person, new_city: String) -> Person {
    let addr_lens = person_address_lens();
    let city_lens = address_city_lens();

    let new_address = city_lens.update(addr_lens.view(&person), new_city);
    addr_lens.update(person, new_address)
}

fn main() {
    let person = Person {
        name: "Bob".to_string(),
        age: 25,
        address: Address {
            city: "Nagoya".to_string(),
            zip: "460-0001".to_string(),
        },
    };

    let updated = set_city(person, "Sapporo".to_string());
    println!("{}", updated.address.city); // "Sapporo"
}
```

---

## Optics の全体像

Lens は **Optics** と呼ばれる抽象群の一部です。

| 名前 | 対象 | 操作の対象数 | 用途 |
|------|------|------------|------|
| **Lens** | 積型（struct） | 必ず1つ | struct のフィールドアクセス |
| **Prism** | 和型（enum） | 0か1つ | enum のバリアント取り出し |
| **Iso** | 同型な型 | 必ず1つ | 型変換（可逆） |
| **Traversal** | コレクション | 0以上 | 複数要素への一括アクセス |

### Prism の例

Prism は enum のバリアントに対して「もしそのバリアントなら取り出せる」操作を提供します。

```rust
#[derive(Debug, Clone)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

struct Prism<S, A> {
    preview: fn(&S) -> Option<&A>,
    review: fn(A) -> S,
}

// Shape::Circle の radius を取り出す Prism は
// preview が Option を返すのが特徴
fn circle_radius_prism() -> impl Fn(&Shape) -> Option<f64> {
    |shape| match shape {
        Shape::Circle { radius } => Some(*radius),
        _ => None,
    }
}

fn main() {
    let shapes = vec![
        Shape::Circle { radius: 5.0 },
        Shape::Rectangle { width: 3.0, height: 4.0 },
        Shape::Circle { radius: 2.0 },
    ];

    // Circle の radius だけを取り出す
    let radii: Vec<f64> = shapes.iter()
        .filter_map(circle_radius_prism())
        .collect();

    println!("{:?}", radii); // [5.0, 2.0]
}
```

---

## 実用的なパターン: derive による自動生成

実際のプロジェクトでは [`lens-rs`](https://crates.io/crates/lens-rs) などのクレートが Lens を derive マクロで自動生成します。

```toml
# Cargo.toml（参考）
[dependencies]
lens-rs = "0.3"
```

```rust
// derive マクロを使うイメージ（lens-rs クレートの場合）
#[derive(Lens)]
struct Person {
    #[lens(name = "person_name")]
    name: String,
    address: Address,
}
```

しかし Rust の型システムの制約から、Haskell の `lens` ライブラリほど強力な合成は難しいため、実用上は「更新関数をシンプルに定義する」アプローチが多く取られます。

---

## まとめ

| 概念 | 役割 |
|------|------|
| Lens | struct のフィールドへの get/set を一級の値として扱う |
| Prism | enum のバリアントへの条件付きアクセス |
| Traversal | コレクション全体への一括操作 |
| 合成 | 複数の Lens を繋げてネストした値にアクセス |

Lens/Optics は**不変データ構造を関数型スタイルで操作する**ための強力な抽象です。Rust では言語制約から完全な実装は難しいですが、概念を理解することで設計力が上がります。

---

## よくある落とし穴と対処法

**落とし穴1: ライフタイムの複雑化**

Lens を `fn(&S) -> &A` で定義するとライフタイムが複雑になります。

```rust
// NG: ライフタイムエラーになりやすい
struct Lens<S, A> {
    get: fn(&S) -> &A, // これはライフタイムパラメータが必要
}
```

**対処:** `get` は `&A` の代わりに `A: Clone` として `A` を返すか、クロージャをジェネリクスで表現します。

**落とし穴2: Lens の合成でボックスが増える**

複数の Lens を合成すると `Box<dyn Fn>` が積み重なりパフォーマンスに影響します。

**対処:** 合成 Lens を関数として直接定義するか、マクロで生成します。

---

## 章末演習問題

1. `Company { name: String, ceo: Person }` という型を定義し、`Company → ceo → address → city` を辿って都市名を取得・更新する関数を作ってください。

2. 以下の enum に対して、`Ok` バリアントの値を `f64` から `i64` に変換する Prism 的な関数を実装してください。
```rust
enum Value {
    Int(i64),
    Float(f64),
    Text(String),
}
```

3. `Vec<Person>` の全員の `age` に 1 を加える Traversal 的な関数を実装してください（イテレータと Lens を組み合わせる）。
