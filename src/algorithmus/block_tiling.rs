use std::thread; // standard library threading API
use core_affinity::{CoreId, set_for_current}; // crate to bind threads to specific CPU cores

/// Multi‑threaded matrix multiplication with cache‑friendly *block tiling*.
/// All important details are now commented **line‑by‑line** so you can follow
/// the control‑flow and memory accesses precisely.
///
/// * `a`, `b` – immutable square `n×n` input matrices in *row‑major* layout
/// * `c`      – mutable output matrix (pre‑allocated to `n×n`)
/// * `n`      – dimension of the matrices
/// * `threads`– number of worker threads to spawn
/// * `pinnen` – list of CPU cores to pin the workers to (≥ `threads` entries)
/// * `block`  – edge length of the square tile (choose a power of two that
///              fits your L1/L2 cache, e.g. 32 or 64)
pub fn tiling(
    a: &Vec<Vec<u32>>,     // reference to matrix A (Vec of rows → Vec<u32>)
    b: &Vec<Vec<u32>>,     // reference to matrix B
    c: &mut Vec<Vec<u32>>, // mutable reference to output matrix C
    n: usize,              // side length of the square matrices
    threads: usize,        // number of OS threads to spawn
    pinnen: &Vec<CoreId>,  // CPU cores for thread affinity
) { // ───────────────────── function body begins ────────────────
    // We spawn a *scoped* thread pool so borrows of `a`, `b`, `c` live long
    // enough and no `static` lifetime gymnastics are needed.
    thread::scope(|s| {
        let mut remaining_rows = c.as_mut_slice(); // slice of rows not yet assigned to a worker
        let mut global_row_offset = 0;             // index of the first row inside `remaining_rows`

        let base_rows  = n / threads; // how many rows each worker gets *at minimum*
        let extra_rows = n % threads; // leftover rows distributed to the first `extra_rows` workers

        for t in 0..threads { // iterate over worker indices
            let rows_here = if t < extra_rows { base_rows + 1 } else { base_rows }; // rows for this worker
            let (my_rows, rest) = remaining_rows.split_at_mut(rows_here); // cut my slice out of the remaining rows
            let start_i = global_row_offset; // absolute row index of my first row in C (and thus A)
            let core    = pinnen[t];         // CPU core to pin this worker to
            let block = 32;

            // Spawn the worker thread – move ownership of `my_rows` etc. inside.
            s.spawn(move || {
                set_for_current(core); // pin thread to its designated core for better cache locality

                // Iterate over my *local* rows: 0..rows_here maps to absolute row `i` via `start_i`.
                for local_i in 0..rows_here {
                    let i       = start_i + local_i; // absolute row index in A and C
                    let row_c   = &mut my_rows[local_i]; // mutable reference to the output row in C

                    // Iterate over *j‑blocks* (column tiles) along the row.
                    for j_block in (0..n).step_by(block) { // start index of the j‑tile
                        let j_end = (j_block + block).min(n); // exclusive upper bound of this j‑tile

                        // Iterate over *columns j* inside this j‑block.
                        for j in j_block..j_end {
                            let mut sum = 0u32; // accumulator for C[i][j]

                            // Iterate over *k‑blocks* (tiles along the inner dimension).
                            for k_block in (0..n).step_by(block) {
                                let k_end = (k_block + block).min(n); // exclusive upper bound of k‑tile

                                // Accumulate the partial dot‑product of the corresponding
                                // segments of row `i` of A and column `j` of B.
                                for k in k_block..k_end {
                                    sum += a[i][k] * b[k][j]; // multiply‑add → systolic style
                                }
                            }
                            row_c[j] = sum; // write the computed element to C
                        }
                    }
                }
            }); // worker spawned – closure ends here

            remaining_rows = rest;         // drop the rows we just assigned
            global_row_offset += rows_here; // advance absolute row offset for next worker
        }
    }); // all workers have joined when the scope ends – safe to return
} // ───────────────────────── function ends ─────────────────────