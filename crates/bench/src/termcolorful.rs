#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum Color {
  Red,
  Green,
  Black,
  Yellow,
  Blue,
  Purple,
  Cyan,
  White,
}
impl std::fmt::Display for Color {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      Color::Black => write!(f, "30"),
      Color::Red => write!(f, "31"),
      Color::Green => write!(f, "32"),
      Color::Yellow => write!(f, "33"),
      Color::Blue => write!(f, "34"),
      Color::Purple => write!(f, "35"),
      Color::Cyan => write!(f, "36"),
      Color::White => write!(f, "37"),
    }
  }
}
pub fn println_string_with_fg_color<T: AsRef<str>>(content: T, color: Color) {
  println!("\x1b[0;{}m{}\x1b[0m", color, content.as_ref());
}
