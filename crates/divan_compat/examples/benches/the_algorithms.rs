use codspeed_divan_compat_examples::the_algorithms;
use divan::black_box;

fn main() {
    divan::main();
}

mod backtracking {
    use super::*;

    #[divan::bench(args = [4, 5, 6, 7, 8])]
    fn n_queens_solver(n: usize) -> Vec<Vec<String>> {
        println!("n_queens_solver for n = {}", n);
        the_algorithms::backtracking::n_queens_solver(n)
    }

    // Benchmark parentheses generation with different sizes
    #[divan::bench(args = [3, 4, 5, 6])]
    fn generate_parentheses(n: usize) -> Vec<String> {
        println!("generate_parentheses for n = {}", n);
        the_algorithms::backtracking::generate_parentheses(n)
    }

    // Benchmark combinations generation with different n values, keeping k=3
    #[divan::bench(args = [5, 6, 7, 8, 9])]
    fn generate_combinations(n: usize) -> Vec<Vec<usize>> {
        println!("generate_combinations for n = {}", n);
        the_algorithms::backtracking::generate_all_combinations(n, 3).unwrap()
    }

    // Benchmark graph coloring with different sizes of complete graphs
    #[divan::bench(args = [3, 4, 5, 6])]
    fn graph_coloring(bencher: divan::Bencher, n: usize) {
        println!("graph_coloring for n = {}", n);
        // Create a complete graph of size n (all vertices connected to each other)
        let matrix = (0..n)
            .map(|i| (0..n).map(|j| i != j).collect())
            .collect::<Vec<Vec<bool>>>();

        bencher.bench_local(|| {
            black_box(the_algorithms::backtracking::generate_colorings(
                black_box(matrix.clone()),
                3,
            ))
        });
    }

    // Benchmark Hamiltonian cycle finding with different sizes of cyclic graphs
    #[divan::bench(args = [4, 5, 6, 7])]
    fn hamiltonian_cycle(bencher: divan::Bencher, n: usize) {
        println!("hamiltonian_cycle for n = {}", n);
        // Create a cyclic graph where each vertex is connected to its neighbors
        // This ensures a Hamiltonian cycle exists
        let matrix = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        let prev = (i + n - 1) % n;
                        let next = (i + 1) % n;
                        j == prev || j == next
                    })
                    .collect()
            })
            .collect::<Vec<Vec<bool>>>();

        bencher.bench_local(|| {
            black_box(the_algorithms::backtracking::find_hamiltonian_cycle(
                black_box(matrix.clone()),
                0,
            ))
        });
    }

    // Benchmark Knight's Tour with different board sizes
    #[divan::bench(args = [5, 6, 7, 8])]
    fn knight_tour(bencher: divan::Bencher, n: usize) {
        println!("knight_tour for n = {}", n);
        bencher.bench_local(|| {
            black_box(the_algorithms::backtracking::find_knight_tour(
                black_box(n),
                black_box(n),
                black_box(0),
                black_box(0),
            ))
        });
    }

    // Benchmark permutations with different input sizes
    #[divan::bench(args = [3, 4, 5, 6, 7])]
    fn permutations(bencher: divan::Bencher, n: usize) {
        println!("permutations for n = {}", n);
        let nums: Vec<isize> = (0..n).map(|x| x as isize).collect();

        bencher.bench_local(|| {
            black_box(the_algorithms::backtracking::permute(black_box(
                nums.clone(),
            )))
        });
    }

    // Benchmark Rat in Maze with different maze sizes
    #[divan::bench(args = [5, 6, 7, 8])]
    fn rat_in_maze(bencher: divan::Bencher, n: usize) {
        println!("rat_in_maze for n = {}", n);
        // Create a maze where the rat can move diagonally to the end
        let maze = (0..n)
            .map(|i| (0..n).map(|j| i == j || i == j + 1).collect())
            .collect::<Vec<Vec<bool>>>();

        bencher.bench_local(|| {
            black_box(the_algorithms::backtracking::find_path_in_maze(
                black_box(&maze),
                black_box(0),
                black_box(0),
            ))
        });
    }

    // Benchmark Subset Sum with different set sizes
    #[divan::bench(args = [10, 12, 14, 16, 18])]
    fn subset_sum(bencher: divan::Bencher, n: usize) {
        println!("subset_sum for n = {}", n);
        let set: Vec<isize> = (0..n).map(|x| x as isize).collect();
        let target = (n as isize) * 2; // A challenging but achievable target

        bencher.bench_local(|| {
            black_box(the_algorithms::backtracking::has_subset_with_sum(
                black_box(&set),
                black_box(target),
            ))
        });
    }

    // Benchmark Sudoku solver with different levels of difficulty
    #[divan::bench]
    fn sudoku(bencher: divan::Bencher) {
        println!("sudoku solver benchmark");
        // A moderately difficult Sudoku puzzle
        let board = [
            [3, 0, 6, 5, 0, 8, 4, 0, 0],
            [5, 2, 0, 0, 0, 0, 0, 0, 0],
            [0, 8, 7, 0, 0, 0, 0, 3, 1],
            [0, 0, 3, 0, 1, 0, 0, 8, 0],
            [9, 0, 0, 8, 6, 3, 0, 0, 5],
            [0, 5, 0, 0, 9, 0, 6, 0, 0],
            [1, 3, 0, 0, 0, 0, 2, 5, 0],
            [0, 0, 0, 0, 0, 0, 0, 7, 4],
            [0, 0, 5, 2, 0, 6, 3, 0, 0],
        ];

        bencher.bench_local(|| {
            black_box(the_algorithms::backtracking::sudoku_solver(black_box(
                &board,
            )))
        });
    }
}

// mod bit_manipulation {
//     use super::*;

//     #[divan::bench(args = [0, 42, 255, 1024, 65535])]
//     fn count_set_bits(bencher: divan::Bencher, n: u32) {
//         bencher.bench_local(|| {
//             black_box(the_algorithms::bit_manipulation::count_set_bits(black_box(
//                 n.try_into().unwrap(),
//             )))
//         });
//     }

//     #[divan::bench(args = [0, 42, 255, 1024, 65535])]
//     fn find_highest_set_bit(bencher: divan::Bencher, n: u32) {
//         bencher.bench_local(|| {
//             black_box(the_algorithms::bit_manipulation::find_highest_set_bit(
//                 black_box(n.try_into().unwrap()),
//             ))
//         });
//     }

//     #[divan::bench(args = [1, 2, 3, 4, 5])]
//     fn generate_gray_code(bencher: divan::Bencher, n: u32) {
//         bencher.bench_local(|| {
//             black_box(the_algorithms::bit_manipulation::generate_gray_code(
//                 black_box(n.try_into().unwrap()),
//             ))
//         });
//     }

//     #[divan::bench(args = &[(0, 0), (42, 13), (255, 255), (1024, -1024), (65535, -65535)])]
//     fn add_two_integers(bencher: divan::Bencher, (a, b): (i32, i32)) {
//         bencher.bench_local(|| {
//             black_box(the_algorithms::bit_manipulation::add_two_integers(
//                 black_box(a.try_into().unwrap()),
//                 black_box(b.try_into().unwrap()),
//             ))
//         });
//     }
// }
