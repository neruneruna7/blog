// HTMLではTypstのgrid要素を使えないため，CSS Grid用のdivへ変換する．
// columns/rowsは，整数またはTypstのトラックサイズ配列を受け取る．
// grid.cellによるセル結合や個別配置は，HTML側に対応する情報を出力しない．
#let css-value(value) = if type(value) == relative {
  "calc(" + repr(value) + ")"
} else {
  repr(value)
}

#let my-grid(
  columns: (),
  rows: (),
  gutter: (),
  column-gutter: (),
  row-gutter: (),
  ..cells,
) = context {
  if target() == "html" {
    let column-template = if columns == () {
      "none"
    } else if type(columns) == int {
      assert(columns > 0, message: "my-gridのcolumnsは1以上でなければならない")
      "repeat(" + css-value(columns) + ", minmax(0, 1fr))"
    } else if type(columns) == array {
      assert(columns.len() > 0, message: "my-gridのcolumnsは空にできない")
      columns.map(css-value).join(" ")
    } else {
      css-value(columns)
    }

    let row-template = if rows == () {
      "none"
    } else if type(rows) == int {
      assert(rows > 0, message: "my-gridのrowsは1以上でなければならない")
      "repeat(" + css-value(rows) + ", auto)"
    } else if type(rows) == array {
      assert(rows.len() > 0, message: "my-gridのrowsは空にできない")
      rows.map(css-value).join(" ")
    } else {
      css-value(rows)
    }

    let column-gap = if column-gutter == () {
      gutter
    } else {
      column-gutter
    }
    let row-gap = if row-gutter == () {
      gutter
    } else {
      row-gutter
    }

    let style = (
      "grid-template-columns: " + column-template + "; "
      + "grid-template-rows: " + row-template + "; "
      + if column-gap != () {
          assert(
            type(column-gap) != array,
            message: "HTML出力のmy-gridではcolumn-gutterに配列を指定できない",
          )
          "column-gap: " + css-value(column-gap) + "; "
        } else {
          ""
        }
      + if row-gap != () {
          assert(
            type(row-gap) != array,
            message: "HTML出力のmy-gridではrow-gutterに配列を指定できない",
          )
          "row-gap: " + css-value(row-gap) + ";"
        } else {
          ""
        }
    )

    html.elem(
      "div",
      attrs: (
        class: "my-grid",
        style: style,
      ),
    )[
      #for cell in cells.pos() {
        html.elem(
          "div",
          attrs: (class: "my-grid__cell"),
        )[#cell]
      }
    ]
  } else {
    grid(
      columns: columns,
      rows: rows,
      gutter: gutter,
      column-gutter: column-gutter,
      row-gutter: row-gutter,
      ..cells,
    )
  }
}
