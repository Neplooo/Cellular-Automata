use array2d::{Array2D, Error};

/**
 * Starts the grid in a random state of 0s and 1s.
 */
pub fn initialize_grid(grid_size: usize, density_coef: i8) -> Array2D<i32> {
    
    let mut start_grid = Array2D::filled_with(0, grid_size, grid_size);

    for row in 0..grid_size {
        for col in 0..grid_size {
            let random_val: i32 = match rand::random() {
            0i8 => 0, //If Val is less than 0, it is dead.
            i if i > 0 + density_coef => 1, //If Val is positive and greater than the density coefficient, it is alive.
            _ => 0 //In any other case (negative values), it is dead.
            };
            start_grid.set(row, col, random_val).ok();
        }
    };

    start_grid
}

/**
 * Checks all 8 neighbors of a cell to see how many are alive (1). Returns the number of alive neighbors.
 */
pub fn check_alive_neighbors(grid: &Array2D<i32>, row: usize, col: usize) -> i32 {

    let mut alive_neighbors = 0;
    
    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 {
                continue; //Don't look at the cell itself.
            }

            if let neighbor = grid.get((row as isize + i) as usize, (col as isize + j) as usize) {
                match neighbor {
                    Some(1) => alive_neighbors += 1,
                    _ => continue
                }
            }
        }
    }

    alive_neighbors
}

/**
 * Displays the grid in the console, with alive cells represented by "*" and dead cells represented by " ".
 */
pub fn display_grid(grid: &Array2D<i32>, grid_size: usize, generation: i32) {
    println!("Generation: {}", generation);
    for row in 0..grid_size {
        for col in 0..grid_size {
            print!( "{}", match grid.get(row, col) {
                Some(0) => " ",
                Some(1) => "* ",
                _ => " "
            })
        }
        println!();
    }
}