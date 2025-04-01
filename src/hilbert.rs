use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
struct GridTurtle {
    x: i32,
    y: i32,
    dir: Direction,
}

#[derive(Component, Debug)]
pub struct Hilbert {
    pub forward: Vec<(usize, usize)>,
    pub backward: Vec<Vec<usize>>,
    pub iterations: u32,
    pub dim_size: usize,
    pub total_size: usize,
}

struct HilbertBuilder {
    turtle: GridTurtle,
    hilbert: Hilbert,
    position: usize,
}

impl Direction {
    pub fn turn_left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    pub fn turn_right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    pub fn offsets(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

impl GridTurtle {
    pub fn new(x: i32, y: i32, dir: Direction) -> Self {
        Self { x, y, dir }
    }

    pub fn turn_left(&mut self, invert: bool) {
        self.dir = match invert {
            true => self.dir.turn_right(),
            false => self.dir.turn_left(),
        }
    }

    pub fn turn_right(&mut self, invert: bool) {
        self.dir = match invert {
            true => self.dir.turn_left(),
            false => self.dir.turn_right(),
        }
    }

    pub fn forward(&mut self) {
        let (x_off, y_off) = self.dir.offsets();
        self.x += x_off;
        self.y += y_off;
        let (x, y) = (self.x, self.y);
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }
}

impl Hilbert {
    pub fn new(iterations: u32) -> Self {
        HilbertBuilder::build(iterations)
    }
}

impl HilbertBuilder {
    fn build(iterations: u32) -> Hilbert {
        let dim_size = (2 as usize).pow(iterations);
        let total_size = dim_size * dim_size;
        let mut forward = Vec::with_capacity(total_size);
        unsafe { forward.set_len(total_size) }
        let mut backward: Vec<Vec<_>> = Vec::with_capacity(dim_size);
        for i in 0..dim_size {
            backward.push(Vec::with_capacity(dim_size));
            unsafe { backward[i].set_len(dim_size) }
        }

        let mut turtle = GridTurtle::new(0, 0, Direction::Up);
        let mut hilbert = Hilbert {
            forward,
            backward,
            iterations,
            dim_size,
            total_size,
        };

        hilbert.forward[0] = (0, 0);
        hilbert.backward[0][0] = 0;

        let mut builder = HilbertBuilder {
            turtle,
            hilbert,
            position: 1,
        };

        builder.iterate(iterations, false);
        builder.hilbert
    }

    fn iterate(&mut self, level: u32, invert: bool) {
        if level == 0 {
            return;
        }

        self.turtle.turn_right(invert);
        self.iterate(level - 1, !invert);

        self.turtle.forward();
        let (x, y) = self.turtle.pos();
        self.hilbert.forward[self.position] = (x as usize, y as usize);
        self.hilbert.backward[x as usize][y as usize] = self.position;
        self.position += 1;

        self.turtle.turn_left(invert);
        self.iterate(level - 1, invert);

        self.turtle.forward();
        let (x, y) = self.turtle.pos();
        self.hilbert.forward[self.position] = (x as usize, y as usize);
        self.hilbert.backward[x as usize][y as usize] = self.position;
        self.position += 1;

        self.iterate(level - 1, invert);

        self.turtle.turn_left(invert);

        self.turtle.forward();

        let (x, y) = self.turtle.pos();
        self.hilbert.forward[self.position] = (x as usize, y as usize);
        self.hilbert.backward[x as usize][y as usize] = self.position;
        self.position += 1;

        self.iterate(level - 1, !invert);
        self.turtle.turn_right(invert);
    }
}
