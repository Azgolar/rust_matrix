use rayon::iter::{ParallelIterator, IndexedParallelIterator, IntoParallelRefMutIterator};

/*
    Parallele Matrixmultiplikation mit Rayon
    CPU Pinning erfolgt beim erstellen des Thread Pools in benchmark.rs
*/
pub fn parallel(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize) {

    c.par_iter_mut().enumerate().for_each(|(i, zeile): (usize, &mut Vec<u32>)| {
        for j in 0..n {
            let mut summe: u32 = 0;
            for k in 0..n {
                summe = summe + a[i][k] * b[k][j];
            }
            zeile[j] = summe;
        }
    });
}