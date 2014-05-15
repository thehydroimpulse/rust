pub struct Filter<'a> {
  func: |stream: &'a str|:'a -> StrBuf
}
