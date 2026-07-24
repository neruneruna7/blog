// 記事本文をここに記述する．

= コード
このブログは次のtypstコードによって書かれている．
```typc
// 各数の読み方
#let aho-digits = (
  "1": "ｲｨﾁ",
  "2": "ﾆ",
  "3": "ｻｧﾝ",
  "4": "ｼ",
  "5": "ｺﾞ",
  "6": "ﾛｸ",
  "7": "ﾅﾅ",
  "8": "ﾊﾁ",
  "9": "ｷｭｰ",
)

// 桁ごとの読み方
#let aho-places = (
  "10000": (
    read: "ﾏﾝ",
    omit-one: false,
  ),
  "1000": (
    read: "ｾﾝ",
    omit-one: true,
  ),
  "100": (
    read: "ﾋｬｸ",
    omit-one: true,
  ),
  "10": (
    read: "ｼﾞｭｳ",
    omit-one: true,
  ),
  "1": (
    read: "",
    omit-one: false,
  ),
)

#let aho-read(n) = {
  assert(
    n >= 1 and n <= 99999,
    message: "1〜99999の自然数にのみ対応",
  )

  let rest = n
  let result = ""

  for (place-key, rule) in aho-places {
    let place = int(place-key)
    let digit = calc.div-euclid(rest, place)

    rest = calc.rem-euclid(rest, place)

    if digit > 0 {
      // 10・100・1000 ... では先頭の「1」を省略する
      if digit != 1 or not rule.omit-one {
        result += aho-digits.at(str(digit))
      }

      result += rule.read
    }
  }

  result
}

#let is-aho(n) = {
  calc.rem-euclid(n, 3) == 0 or str(n).contains("3")
}

#let nabeatsu(n) = {
  if is-aho(n) {
    aho-read(n)
  } else {
    str(n)
  }
}
= ナベアツ

#let i = 1
#while i < 400 {
  [
  #nabeatsu(i),
  ]
  i += 1
}
```
#let aho-digits = (
  "1": "ｲﾁ",
  "2": "ﾆ",
  "3": "ｻｧﾝ",
  "4": "ﾖﾝ",
  "5": "ｺﾞ",
  "6": "ﾛｸ",
  "7": "ﾅﾅ",
  "8": "ﾊﾁ",
  "9": "ｷｭｰ",
)

// 大きい位から定義する
#let aho-places = (
  "10000": (
    read: "ﾏﾝ",
    omit-one: false,
  ),
  "1000": (
    read: "ｾﾝ",
    omit-one: true,
  ),
  "100": (
    read: "ﾋｬｸ",
    omit-one: true,
  ),
  "10": (
    read: "ｼﾞｭ-",
    omit-one: true,
  ),
  "1": (
    read: "",
    omit-one: false,
  ),
)

#let aho-read(n) = {
  assert(
    n >= 1 and n <= 99999,
    message: "1〜99999の自然数を指定せよ",
  )

  let rest = n
  let result = ""

  for (place-key, rule) in aho-places {
    let place = int(place-key)
    let digit = calc.div-euclid(rest, place)

    rest = calc.rem-euclid(rest, place)

    if digit > 0 {
      // 十・百・千では先頭の「一」を省略する
      if digit != 1 or not rule.omit-one {
        result += aho-digits.at(str(digit))
      }
      result += rule.read
    }
  }

  result
}

#let is-aho(n) = {
  calc.rem-euclid(n, 3) == 0 or str(n).contains("3")
}

#let nabeatsu(n) = {
  if is-aho(n) {
    [#aho-read(n)www]
  } else {
    str(n)
  }
}
= ナベアツ本文

#let i = 1
#while i < 400 {
  [
  #nabeatsu(i),
  ]
  i += 1
}
