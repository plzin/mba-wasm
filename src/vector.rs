use std::boxed::Box;
use std::ops::{Index, IndexMut, AddAssign, Mul};
use std::fmt::Write;

use num_traits::Num;

use crate::numbers::UnsignedInt;

#[derive(Clone)]
pub struct Vector<T> {
    /// The number of entries in the vector.
    pub dim: usize,

    /// Memory that holds the entries.
    pub(self) entries: Box<[T]>,
}

impl<T> Vector<T> {
    /// Returns an empty vector.
    pub fn empty() -> Self {
        Self {
            dim: 0,
            entries: Box::new([]),
        }
    }

    /// Is the vector empty.
    pub fn is_empty(&self) -> bool {
        self.dim == 0
    }

    /// Creates a vector from a slice.
    pub fn from_slice<U: Clone>(s: &[U]) -> Self
        where T: From<U>,
     {
        Self {
            dim: s.len(),
            entries: s.iter()
                .map(|e| T::from(e.clone()))
                .collect::<Vec<_>>()
                .into_boxed_slice()
        }
    }

    /// Returns a slice of the entries.
    pub fn entries(&self) -> &[T] {
        &self.entries
    }

    /// Returns a mutable slice of the entries.
    pub fn entries_mut(&mut self) -> &mut [T] {
        &mut self.entries
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.entries().iter()
    }

    /// Returns an iterator over mutable references to the elements.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.entries_mut().iter_mut()
    }

    /// Returns an reference to an entry.
    pub fn entry(&self, i: usize) -> &T {
        &self.entries[i]
    }

    /// Returns a mutable reference to an entry.
    pub fn entry_mut(&mut self, i: usize) -> &mut T {
        &mut self.entries[i]
    }

    /// Apply a function to each entry.
    pub fn map<U, F>(&self, mut f: F) -> Vector<U>
        where F: FnMut(&T) -> U
    {
        let mut v = Vec::with_capacity(self.entries.len());
        for e in self.entries.iter() {
            v.push(f(e));
        }
        
        Vector::<U> {
            dim: self.dim,
            entries: v.into_boxed_slice()
        }
    }

    /// Apply a function to each entry that can fail.
    /// If the function fails for the first time, then that resulting error
    /// and the location of the entry is returned.
    pub fn try_map<U, E, F>(&self, mut f: F) -> Result<Vector<U>, (usize, E)>
        where F: FnMut(&T) -> Result<U, E>
    {
        let mut v = Vec::with_capacity(self.entries.len());
        for e in self.entries.iter() {
            match f(e) {
                Ok(e) => v.push(e),
                Err(e) => {
                    return Err((v.len(), e));
                }
            }
        }
        
        Ok(Vector::<U> {
            dim: self.dim,
            entries: v.into_boxed_slice()
        })
    }
}

impl<T: Clone> Vector<T> {
    /// Creates a new vector whose entries are initialized with `val`.
    pub fn uniform(dim: usize, val: T) -> Self {
        Self {
            dim,
            entries: vec![val; dim].into_boxed_slice(),
        }
    }
}

impl<T: Num + Clone> Vector<T> {
    /// Returns a zero vector.
    pub fn zero(dim: usize) -> Self {
        Self {
            dim,
            entries: vec![T::zero(); dim].into_boxed_slice(),
        }
    }

    /// Is this the zero vector?
    pub fn is_zero(&self) -> bool {
        self.iter().all(|e| *e == T::zero())
    }
}

impl<T: UnsignedInt> Vector<T> {
    pub fn to_tex(&self) -> String {
        let mut s = "\\left[\\begin{array}{}".to_owned();
        for e in self.iter().cloned() {
            if (e.print_negative()) {
                write!(&mut s, "-{}\\\\", T::zero() - e);
            } else {
                write!(&mut s, "{}\\\\", e);
            }
        }
        s += "\\end{array}\\right]";
        s
    }
}

impl<T> Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.entry(index)
    }
}

impl<T> IndexMut<usize> for Vector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.entry_mut(index)
    }
}

impl<T: Num + Copy> Mul<T> for Vector<T> {
    type Output = Vector<T>;
    fn mul(mut self, rhs: T) -> Self::Output {
        for e in self.iter_mut() {
            *e = *e * rhs;
        }
        self
    }
}

impl<T: Num + Copy> AddAssign<&Vector<T>> for Vector<T> {
    fn add_assign(&mut self, rhs: &Vector<T>) {
        debug_assert!(self.dim == rhs.dim);
        for (e, f) in self.iter_mut().zip(rhs.iter()) {
            *e = *e + *f;
        }
    }
}