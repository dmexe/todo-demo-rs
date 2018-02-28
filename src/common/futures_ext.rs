use futures::Future;

pub trait FuturesExt: Future {
  fn into_boxed(self) -> Box<Self>;
}

impl<F, T, E> FuturesExt for F
where
  F: Future<Item = T, Error = E> + Sized + 'static,
{
  fn into_boxed(self) -> Box<Self> {
    Box::new(self)
  }
}
