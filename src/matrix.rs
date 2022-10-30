use std::boxed::Box;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Mul};
use std::fmt::Write;
use num_traits::{Num, NumAssign};
use crate::congruence_solver::ModN;
use crate::vector::Vector;

#[derive(Clone)]
pub struct Matrix<T> {
    /// The number of rows.
    pub rows: usize,

    /// The number of column.
    pub cols: usize,

    /// Memory that holds the entries.
    pub(self) entries: Box<[T]>,
}

impl<T> Matrix<T> {
    /// Return an empty matrix.
    pub fn empty() -> Self {
        Self {
            rows: 0,
            cols: 0,
            entries: Box::new([]),
        }
    }

    /// Create a matrix from an array.
    pub fn from_array<U: Into<T>, const R: usize, const C: usize>(
        a: [[U; C]; R]
    ) -> Self {
        let entries = a.into_iter()
            .flatten()
            .map(|e| e.into())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            rows: R,
            cols: C,
            entries,
        }
    }

    /// Returns the minimum of the two dimension.
    pub fn min_dim(&self) -> usize {
        std::cmp::min(self.rows, self.cols)
    }

    /// Returns a reference to an entry.
    pub fn entry(&self, r: usize, c: usize) -> &T {
        &self.entries[r * self.cols + c]
    }

    /// Returns a mutable reference to an entry.
    pub fn entry_mut(&mut self, r: usize, c: usize) -> &mut T {
        &mut self.entries[r * self.cols + c]
    }

    /// Returns a pointer to an entry.
    pub fn entry_ptr(&mut self, r: usize, c: usize) -> *mut T {
        self.entry_mut(r, c) as *mut T
    }

    /// Returns a row as a slice.
    pub fn row(&self, r: usize) -> &[T] {
        let i = r * self.cols;
        &self.entries[i..i+self.cols]
    }

    /// Returns a row as a mutable slice.
    pub fn row_mut(&mut self, r: usize) -> &mut [T] {
        let i = r * self.cols;
        &mut self.entries[i..i+self.cols]
    }

    /// Returns an iterator over the column.
    pub fn col(&self, c: usize) -> Column<T> {
        Column::from_matrix(self, c)
    }

    /// Returns an iterator over mutable references to the elements in the column.
    pub fn col_mut(&mut self, c: usize) -> ColumnMut<T> {
        ColumnMut::from_matrix(self, c)
    }

    /// Apply a function to each entry.
    pub fn map<U, F>(&self, mut f: F) -> Matrix<U>
        where F: FnMut(&T) -> U
    {
        let mut v = Vec::with_capacity(self.entries.len());
        for e in self.entries.iter() {
            v.push(f(e));
        }
        
        Matrix::<U> {
            rows: self.rows,
            cols: self.cols,
            entries: v.into_boxed_slice()
        }
    }

    /// Apply a function to each entry that can fail.
    /// If the function for the first time, then that resulting error
    /// and the location of the entry is returned.
    pub fn try_map<U, E, F>(&self, mut f: F) -> Result<Matrix<U>, (usize, usize, E)>
        where F: FnMut(&T) -> Result<U, E>
    {
        let mut v = Vec::with_capacity(self.entries.len());
        for e in self.entries.iter() {
            match f(e) {
                Ok(e) => v.push(e),
                Err(e) => {
                    let row = v.len() / self.rows;
                    let col = v.len() % self.cols;
                    return Err((row, col, e));
                }
            }
        }
        
        Ok(Matrix::<U> {
            rows: self.rows,
            cols: self.cols,
            entries: v.into_boxed_slice()
        })
    }
}

impl<T: Clone> Matrix<T> {
    /// Creates a new matrix whose entries are initialized with `val`.
    pub fn uniform(r: usize, c: usize, val: T) -> Self {
        Self {
            rows: r,
            cols: c,
            entries: vec![val; r*c].into_boxed_slice(),
        }
    }
}

impl<T: NumAssign + Copy> Matrix<T> {
    /// Returns a r×c zero matrix.
    pub fn zero(r: usize, c: usize) -> Self {
        if r == 0 || c == 0 {
            return Self::empty();
        }

        Self {
            rows: r,
            cols: c,
            entries: vec![T::zero(); r*c].into_boxed_slice(),
        }
    }

    /// Returns an n×n identity matrix.
    pub fn id(n: usize) -> Self {
        let mut m = Self::zero(n, n);
        for i in 0..n {
            m[(i, i)] = T::one();
        }
        m
    }

    /// Swap two rows.
    pub fn swap_rows(&mut self, i: usize, j: usize) {
        if i == j { return }

        unsafe {
            core::ptr::swap_nonoverlapping(
                self.entry_ptr(i, 0),
                self.entry_ptr(j, 0), self.cols
            )
        }
    }

    /// Swap two columns.
    pub fn swap_columns(&mut self, i: usize, j: usize) {
        if i == j { return }

        for k in 0..self.rows {
            unsafe {
                core::ptr::swap_nonoverlapping(
                    self.entry_ptr(k, i), self.entry_ptr(k, j), 1
                );
            }
        }
    }

    /// Add a scaled row to another row. N = c * M.
    pub fn row_multiply_add(&mut self, n: usize, m: usize, c: T) {
        for i in 0..self.cols {
            let s = self[(n, i)] * c;
            self[(m, i)] += s;
        }
    }

    /// Add a scaled column to another column. N = c * M.
    pub fn col_multiply_add(&mut self, n: usize, m: usize, c: T) {
        for i in 0..self.rows {
            let s = self[(i, n)] * c;
            self[(i, m)] += s;
        }
    }
}

impl<T: ModN> Matrix<T> {
    /// Convert the matrix into a latex renderable string.
    pub fn to_tex(&self) -> String {
        let mut s = "\\left[\\begin{array}{}".to_owned();
        for r in 0..self.rows {
            for e in self.row(r).iter().cloned() {
                if (e.print_negative()) {
                    write!(&mut s, "-{} & ", T::zero() - e);
                } else {
                    write!(&mut s, "{} & ", e);
                }
            }

            // Remove the last " & ".
            s.truncate(s.len() - 3);
            s += "\\\\";
        }

        s += "\\end{array}\\right]";
        s
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (r, c): (usize, usize)) -> &Self::Output {
        self.entry(r, c)
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut Self::Output {
        self.entry_mut(r, c)
    }
}

impl<T: NumAssign + Copy> Mul for &Matrix<T> {
    type Output = Matrix<T>;
    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(self.cols == rhs.rows,
            "Can't multiply matrices because of incompatible dimensions");
        
        let mut m = Self::Output::zero(self.rows, rhs.cols);

        for i in 0..m.rows {
            for j in 0..m.cols {
                m[(i, j)] = self.row(i).iter()
                    .zip(rhs.col(j))
                    .map(|(l, r)| *l * *r)
                    .fold(T::zero(), T::add);
            }
        }

        m
    }
}

impl<T: NumAssign + Copy> Mul<&Vector<T>> for &Matrix<T> {
    type Output = Vector<T>;
    fn mul(self, rhs: &Vector<T>) -> Self::Output {
        debug_assert!(self.cols == rhs.dim,
            "Can't multiply matrix/vector because of incompatible dimensions");

        let mut m = Vector::<T>::zero(self.rows);
        for i in 0..m.dim {
            m[i] = self.row(i).iter()
                .zip(rhs.iter())
                .map(|(l, r)| *l * *r)
                .fold(T::zero(), T::add);
        }

        m
    }
}

impl<T: PartialEq> PartialEq for Matrix<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.cols == other.cols &&
        self.entries.iter()
            .zip(other.entries.iter())
            .all(|(e, f)| *e == *f)
    }
}

pub struct Column<'a, T> {
    ptr: *const T,
    end: *const T,
    off: usize,
    marker: PhantomData<&'a T>,
}

impl<'a, T> Column<'a, T> {
    pub fn from_matrix(mat: &'a Matrix<T>, c: usize) -> Self {
        debug_assert!(c < mat.cols);
        unsafe {
            Self {
                ptr: mat.entries.as_ptr().add(c),
                end: mat.entries.as_ptr().add(mat.entries.len()),
                off: mat.cols,
                marker: PhantomData,
            }
        }
    }
}

impl<'a, T> Iterator for Column<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr < self.end {
            unsafe {
                let e = &*self.ptr;
                self.ptr = self.ptr.add(self.off);
                Some(e)
            }
        } else {
            None
        }
    }
}

pub struct ColumnMut<'a, T> {
    ptr: *mut T,
    end: *mut T,
    off: usize,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T> ColumnMut<'a, T> {
    pub fn from_matrix(mat: &'a mut Matrix<T>, c: usize) -> Self {
        debug_assert!(c < mat.cols);
        unsafe {
            Self {
                ptr: mat.entries.as_mut_ptr().add(c),
                end: mat.entries.as_mut_ptr().add(mat.entries.len()),
                off: mat.cols,
                marker: PhantomData,
            }
        }
    }
}

impl<'a, T> Iterator for ColumnMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr < self.end {
            unsafe {
                let e = &mut *self.ptr;
                self.ptr = self.ptr.add(self.off);
                Some(e)
            }
        } else {
            None
        }
    }
}