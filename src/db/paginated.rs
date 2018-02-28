/// A structure that holds pagination result.
#[derive(Debug, Serialize)]
pub struct Paginated<T> {
  /// a next item's id
  pub next: Option<i64>,
  /// a list of items.
  pub items: Vec<T>,
}
