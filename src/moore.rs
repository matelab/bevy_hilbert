use super::hilbert::Hilbert;
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Moore {
    pub forward: Vec<(usize, usize)>,
    pub backward: Vec<Vec<usize>>,
    pub iterations: u32,
    pub dim_size: usize,
    pub total_size: usize,
}

impl Moore {
    pub fn new(iterations: u32) -> Self {
        assert!(
            iterations > 1,
            "Moore curve requires at least two iterations"
        );

        // Create forwards and backwards arrays
        let dim_size = (2 as usize).pow(iterations);
        let total_size = dim_size * dim_size;
        let mut forward = Vec::with_capacity(total_size);
        unsafe { forward.set_len(total_size) }
        let mut backward: Vec<Vec<_>> = Vec::with_capacity(dim_size);
        for i in 0..dim_size {
            backward.push(Vec::with_capacity(dim_size));
            unsafe { backward[i].set_len(dim_size) }
        }

        // Create base Hilbert curve of iterations-1
        let hilbert = Hilbert::new(iterations - 1);

        // First Hilbert quadrant (Bottom Right)
        let half = dim_size / 2;

        let quadrant = 0;
        for i in 0..hilbert.total_size {
            let (x_src, y_src) = hilbert.forward[i];
            let (x_dst, y_dst) = (half + x_src, y_src);
            forward[quadrant * hilbert.total_size + i] = (x_dst, y_dst);
            backward[x_dst][y_dst] = quadrant * hilbert.total_size + i;
        }

        // Second Hilbert quadrant (Top Right)
        let quadrant = 1;
        for i in 0..hilbert.total_size {
            let (x_src, y_src) = hilbert.forward[i];
            let (x_dst, y_dst) = (half + x_src, half + y_src);
            forward[quadrant * hilbert.total_size + i] = (x_dst, y_dst);
            backward[x_dst][y_dst] = quadrant * hilbert.total_size + i;
        }

        // Third Hilbert quadrant (Top Left)
        let quadrant = 2;
        let quarter = half / 2;
        for i in 0..hilbert.total_size {
            let (x_src, y_src) = hilbert.forward[i];
            let (x_dst, y_dst) = (half - 1 - x_src, (dim_size - 1) - y_src);
            forward[quadrant * hilbert.total_size + i] = (x_dst, y_dst);
            backward[x_dst][y_dst] = quadrant * hilbert.total_size + i;
        }

        // Fourth Hilbert quadrant (Bottom Left)
        let quadrant = 3;
        for i in 0..hilbert.total_size {
            let (x_src, y_src) = hilbert.forward[i];
            let (x_dst, y_dst) = (half - 1 - x_src, half - 1 - y_src);
            forward[quadrant * hilbert.total_size + i] = (x_dst, y_dst);
            backward[x_dst][y_dst] = quadrant * hilbert.total_size + i;
        }

        Moore {
            forward,
            backward,
            iterations,
            dim_size,
            total_size,
        }
    }

    pub fn forward(&self, index: usize) -> Option<(usize, usize)> {
        if index < self.total_size {
            Some(self.forward[index])
        } else {
            None
        }
    }

    pub fn forward_slice(&self, indices: &[usize]) -> Vec<Option<(usize, usize)>> {
        indices.iter().map(|i| self.forward(*i)).collect()
    }

    pub fn forward_field<T>(&self, field: Vec<T>) -> Option<Vec<Vec<T>>> {
        if field.len() == self.total_size {
            let mut grid = Vec::with_capacity(self.dim_size);
            for _ in 0..self.dim_size {
                let mut yes = Vec::with_capacity(self.dim_size);
                unsafe { yes.set_len(self.dim_size) }
                grid.push(yes);
            }

            for (i, t) in field.into_iter().enumerate() {
                let (target_x, target_y) = self.forward(i).unwrap();
                grid[target_x][target_y] = t;
            }
            Some(grid)
        } else {
            None
        }
    }

    pub fn forward_circular(&self, index: isize) -> (usize, usize) {
        let ts = self.total_size as isize;
        let index_circ = ((index % ts) + ts) % ts;
        self.forward(index_circ as usize).unwrap()
    }

    pub fn forward_circular_slice(&self, indices: &[isize]) -> Vec<(usize, usize)> {
        indices.iter().map(|i| self.forward_circular(*i)).collect()
    }

    pub fn backward(&self, index_x: usize, index_y: usize) -> Option<usize> {
        if (index_x < self.dim_size) && (index_y < self.dim_size) {
            Some(self.backward[index_x][index_y])
        } else {
            None
        }
    }

    pub fn backward_slice(&self, indices: &[(usize, usize)]) -> Vec<Option<usize>> {
        indices.iter().map(|(x, y)| self.backward(*x, *y)).collect()
    }

    pub fn backward_grid<T>(&self, grid: Vec<Vec<T>>) -> Vec<T> {
        let mut field = Vec::with_capacity(self.total_size);
        unsafe {
            field.set_len(self.total_size);
        }

        for (index_x, X) in grid.into_iter().enumerate() {
            for (index_y, t) in X.into_iter().enumerate() {
                let idx = self.backward(index_x, index_y).unwrap();
                field[idx] = t;
            }
        }

        field
    }
}
