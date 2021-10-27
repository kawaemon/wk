use std::iter::Map;

pub trait IterExt: Iterator {
    type EditFunc<F>: FnMut(Self::Item) -> Self::Item;

    fn edit<F>(self, f: F) -> Map<Self, Self::EditFunc<F>>
    where
        Self: Sized,
        F: FnMut(&mut Self::Item);
}

impl<T: Iterator> IterExt for T {
    type EditFunc<F> = impl FnMut(Self::Item) -> Self::Item;

    fn edit<F>(self, mut f: F) -> Map<Self, Self::EditFunc<F>>
    where
        Self: Sized,
        F: FnMut(&mut Self::Item),
    {
        self.map(move |mut x| {
            f(&mut x);
            x
        })
    }
}
