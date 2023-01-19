use std::collections::BTreeSet;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Hash)]
pub struct MultiSet<T>(BTreeSet<(T, u8)>);

impl<T: Ord + Clone> FromIterator<T> for MultiSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = BTreeSet::new();

        for v in iter {
            let mut n = 0;
            while !set.insert((v.clone(), n)) {
                n += 1;
            }
        }

        Self(set)
    }
}

impl<T: Ord> MultiSet<T> {
    pub fn into_iter(mut self) -> impl Iterator<Item = T> {
        std::iter::from_fn(move || self.0.pop_first().map(|(t, _)| t))
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().map(|(v, _)| v)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
