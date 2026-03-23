# 第12章: 型状態パターン

## はじめに

**型状態パターン**（Type State Pattern）とは、オブジェクトの状態を型パラメータとして表現し、無効な状態遷移をコンパイル時にエラーにするテクニックです。`PhantomData` を使って実現します。

---

## 基本: PhantomData とは

`PhantomData<T>` は実行時には存在しない（ゼロサイズ）ですが、コンパイラに「型 T を所有/参照している」と伝えることができます。

```rust
use std::marker::PhantomData;

struct Foo<State> {
    data: String,
    _state: PhantomData<State>, // 実行時コストゼロ
}
```

---

## ドアの状態機械

ロックされたドアは開けられない、という制約をコンパイル時に保証します。

```rust
use std::marker::PhantomData;

// 状態を表すマーカー型（データを持たない）
pub struct Locked;
pub struct Unlocked;

pub struct Door<State> {
    name: String,
    _state: PhantomData<State>,
}

impl Door<Locked> {
    pub fn new(name: impl Into<String>) -> Self {
        Door { name: name.into(), _state: PhantomData }
    }

    pub fn unlock(self) -> Door<Unlocked> {
        Door { name: self.name, _state: PhantomData }
    }
}

impl Door<Unlocked> {
    pub fn open(&self) -> String {
        format!("{} is open!", self.name)
    }

    pub fn lock(self) -> Door<Locked> {
        Door { name: self.name, _state: PhantomData }
    }
}
```

使用例:

```rust
let door = Door::new("Front Door");
// door.open(); // コンパイルエラー！Locked 状態に open() は存在しない

let door = door.unlock();
println!("{}", door.open()); // "Front Door is open!"

let door = door.lock();
// door.open(); // 再びコンパイルエラー
```

**利点:** 状態のチェックをランタイムではなくコンパイル時に行うため、バグが混入しない。

---

## ビルダーパターンへの応用

必須フィールドをすべて設定しないと `build()` できないビルダーを作れます。

```rust
pub struct NoEmail;
pub struct HasEmail;
pub struct NoName;
pub struct HasName;

pub struct UserBuilder<E, N> {
    email: Option<String>,
    name: Option<String>,
    _email_state: PhantomData<E>,
    _name_state: PhantomData<N>,
}

impl UserBuilder<NoEmail, NoName> {
    pub fn new() -> Self { /* ... */ }
}

impl<N> UserBuilder<NoEmail, N> {
    pub fn email(self, email: impl Into<String>) -> UserBuilder<HasEmail, N> { /* ... */ }
}

impl<E> UserBuilder<E, NoName> {
    pub fn name(self, name: impl Into<String>) -> UserBuilder<E, HasName> { /* ... */ }
}

// email と name が両方セットされた場合のみ build() できる
impl UserBuilder<HasEmail, HasName> {
    pub fn build(self) -> User { /* ... */ }
}
```

使用例:

```rust
let user = UserBuilder::new()
    .email("alice@example.com")
    .name("Alice")
    .build(); // OK: 両方セット済み

// UserBuilder::new().build(); // コンパイルエラー！email も name もない
// UserBuilder::new().email("x").build(); // コンパイルエラー！name がない
```

フィールドの設定順序は問いません。`name → email` の順でも動作します。

---

## HTTP リクエストの状態機械

`Idle → Pending → Complete` という遷移を型で表現します。

```rust
pub struct Idle;
pub struct Pending;
pub struct Complete;

pub struct Request<State> {
    url: String,
    body: Option<String>,
    response: Option<String>,
    _state: PhantomData<State>,
}

impl Request<Idle> {
    pub fn new(url: impl Into<String>) -> Self { /* ... */ }
    pub fn send(self) -> Request<Pending> { /* ... */ }
}

impl Request<Pending> {
    pub fn receive(self, response: impl Into<String>) -> Request<Complete> { /* ... */ }
}

impl Request<Complete> {
    pub fn response(&self) -> &str { self.response.as_deref().unwrap() }
}
```

使用例:

```rust
let completed = Request::new("https://api.example.com")
    .send()
    .receive("200 OK");

println!("{}", completed.response()); // "200 OK"

// Request::new("url").response() // コンパイルエラー：Idle 状態に response() は存在しない
```

---

## まとめ

型状態パターンのメリット：

| 観点 | 従来（ランタイムチェック） | 型状態パターン |
|------|------------------------|--------------|
| エラー検出 | 実行時 | コンパイル時 |
| パフォーマンス | 条件分岐あり | ゼロコスト |
| ドキュメント | コードに埋もれる | 型シグネチャが仕様 |

`PhantomData` を活用することで、**正しい使い方しかできない API** を設計できます。これは関数型プログラミングの「型による設計」（Making Illegal States Unrepresentable）の実践例です。
