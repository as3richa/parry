use crate::field::Field;
use crate::gf8::Gf8;
use std::fmt;
use std::ops::{Index, IndexMut, Mul};
use std::vec::Vec;

#[derive(Clone, PartialEq)]
struct Matrix<F: Field> {
    pub rows: usize,
    pub columns: usize,
    elements: Box<[F]>,
}

impl<F: Field> Matrix<F> {
    pub fn identity_matrix(rows: usize) -> Self {
        let mut elements = vec![];

        for i in 0..rows {
            for j in 0..i {
                elements.push(F::zero());
            }

            elements.push(F::one());

            for j in i + 1..rows {
                elements.push(F::zero());
            }
        }

        Matrix {
            rows: rows,
            columns: rows,
            elements: elements.into_boxed_slice(),
        }
    }

    pub fn invert(mut self) -> Option<Matrix<F>> {
        assert!(self.rows == self.columns);

        let mut inverse = Matrix::<F>::identity_matrix(self.rows);

        for i in 0..self.rows {
            let mut j = i;

            while j < self.rows && self[j][i] == F::zero() {
                j += 1;
            }

            if j >= self.rows {
                return None;
            }

            if i != j {
                self.swap_rows(i, j);
                inverse.swap_rows(i, j);
            }

            if self[i][i] != F::one() {
                let u = self[i][i];
                for k in 0..self.columns {
                    self[i][k] /= u;
                    inverse[i][k] /= u;
                }
            }

            assert!(self[i][i] == F::one());

            for k in 0..self.rows {
                if k == i {
                    continue;
                }

                if self[k][i] == F::zero() {
                    continue;
                }

                let u = self[k][i];
                for m in 0..self.columns {
                    let v = self[i][m];
                    self[k][m] -= u * v;

                    let r = inverse[i][m];
                    inverse[k][m] -= u * r;
                }

                assert!(self[k][i] == F::zero());
            }
        }

        assert_eq!(self, Matrix::<F>::identity_matrix(self.rows));

        Some(inverse)
    }

    pub fn encoding_matrix(data_shards: usize, parity_shards: usize) -> Matrix<Gf8> {
        let mut matrix = Matrix::<F>::vandermonde_matrix(data_shards, parity_shards);

        for i in 0..matrix.columns {
            if matrix[i][i] == Gf8::zero() {
                let mut j = i + 1;
                while j < matrix.columns {
                    if matrix[i][j] != Gf8::zero() {
                        break;
                    }
                    j += 1;
                }
                assert!(
                    j < matrix.columns,
                    "Unexpectedly missing non-zero element in row {} with column >= {}",
                    i,
                    i
                );

                matrix.swap_columns(i, j);
            }

            assert!(matrix[i][i] != Gf8::zero());

            if matrix[i][i] != Gf8::one() {
                let u = matrix[i][i];
                for row in i + 1..matrix.rows {
                    matrix[row][i] /= u;
                }
                matrix[i][i] = Gf8::one();
            }

            assert!(matrix[i][i] == Gf8::one());

            for j in 0..matrix.columns {
                if i == j {
                    continue;
                }

                if matrix[i][j] == Gf8::zero() {
                    continue;
                }

                let u = matrix[i][j];

                for k in i..matrix.rows {
                    let element = matrix[k][i];
                    matrix[k][j] -= u * element;
                }

                assert!(matrix[i][j] == Gf8::zero());
            }
        }

        matrix
    }

    fn vandermonde_matrix(data_shards: usize, parity_shards: usize) -> Matrix<Gf8> {
        let rows: usize = data_shards + parity_shards;
        let columns: usize = data_shards;
        let mut elements: Vec<Gf8> = Vec::with_capacity(rows * columns);

        for row in 0..rows {
            let mut element = Gf8::one();
            let base = Gf8(row as u8);

            for column in 0..columns {
                elements.push(element);
                element *= base;
            }
        }

        Matrix {
            rows,
            columns,
            elements: elements.into_boxed_slice(),
        }
    }

    fn swap_rows(&mut self, i: usize, j: usize) {
        assert!(i < self.rows);
        assert!(j < self.rows);
        assert!(i != j);

        for column in 0..self.columns {
            let t = self[i][column];
            self[i][column] = self[j][column];
            self[j][column] = t;
        }
    }

    fn swap_columns(&mut self, i: usize, j: usize) {
        assert!(i < self.columns);
        assert!(j < self.columns);
        assert!(i != j);

        for row in 0..self.rows {
            self[row].swap(i, j);
        }
    }
}

impl<F: Field> fmt::Debug for Matrix<F> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "[")?;

        for row in 0..self.rows {
            write!(formatter, "  [")?;

            for column in 0..self.columns {
                if column != 0 {
                    write!(formatter, ", ")?;
                }

                write!(formatter, "{:?}", self[row][column])?;
            }

            writeln!(formatter, "]")?;
        }

        write!(formatter, "]")?;

        Result::Ok(())
    }
}

impl<F: Field> Index<usize> for Matrix<F> {
    type Output = [F];

    fn index(&self, row: usize) -> &[F] {
        &self.elements[row * self.columns..(row + 1) * self.columns]
    }
}

impl<F: Field> IndexMut<usize> for Matrix<F> {
    fn index_mut(&mut self, row: usize) -> &mut [F] {
        &mut self.elements[row * self.columns..(row + 1) * self.columns]
    }
}

impl<F: Field> Mul for Matrix<F> {
    type Output = Matrix<F>;

    fn mul(self, other: Matrix<F>) -> Matrix<F> {
        if self.columns != other.rows {
            panic!("Mismatched matrix dimensions in mul")
        }

        let mut elements: Vec<F> = Vec::with_capacity(self.rows * other.columns);

        for row in 0..self.rows {
            for column in 0..other.columns {
                let mut element = F::zero();

                for i in 0..self.columns {
                    element += self[row][i] * other[i][column]
                }

                elements.push(element);
            }
        }

        Matrix {
            rows: self.rows,
            columns: other.columns,
            elements: elements.into_boxed_slice(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_identity() {
        for i in 0..20 {
            let matrix = Matrix::<Gf8>::identity_matrix(i);
            let inverse_matrix = matrix
                .clone()
                .invert()
                .expect("Identity matrix is invertible");
            assert!(matrix == inverse_matrix);
        }
    }

    #[test]
    fn inverse() {
        let matrix = Matrix {
            rows: 3,
            columns: 3,
            elements: vec![1, 0, 3, 3, 2, 17, 2, 1, 3]
                .into_iter()
                .map(Gf8)
                .collect::<Vec<Gf8>>()
                .into_boxed_slice(),
        };

        let inverse_matrix = matrix.clone().invert().expect("Matrix is invertible");

        assert!(
            matrix.clone() * inverse_matrix.clone() == Matrix::<Gf8>::identity_matrix(3),
            "{:?}",
            matrix * inverse_matrix.clone()
        );
    }

    #[test]
    fn encoding_matrix() {
        let matrix = Matrix::<Gf8>::encoding_matrix(4, 2);
        assert!(
            matrix
                == Matrix {
                    rows: 6,
                    columns: 4,
                    elements: vec![
                        Gf8(0x01u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x01u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x01u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x00u8),
                        Gf8(0x01u8),
                        Gf8(0x1bu8),
                        Gf8(0x1cu8),
                        Gf8(0x12u8),
                        Gf8(0x14u8),
                        Gf8(0x1cu8),
                        Gf8(0x1bu8),
                        Gf8(0x14u8),
                        Gf8(0x12u8),
                    ]
                    .into_boxed_slice()
                },
            "{:?}",
            matrix
        )
    }
}
