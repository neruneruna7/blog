#let blog-rect() = context {
  if target() == "html" {
    html.div(
      style: "
      width: 320px;
      height: 160px;
      background-color: #5b8def;
      border: 2px solid #315ca8;
      border-radius: 12px;
    ",
      role: "img",
      aria-label: "青色の長方形",
    )
  } else {
    [blog-rectはhtml出力専用に実装されている]
  }
}

#blog-rect()
