use core_affinity::{set_for_current, CoreId};

/*
    single threaded Matrixmultiplikation
*/
pub fn starten(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, kern: &CoreId) {

    set_for_current(*kern);

    for i in 0..n {
        for j in 0..n {
            let mut summe: u32 = 0;
            for k in 0..n {
                summe = summe + a[i][k] * b[k][j];
            }
            c[i][j] = summe;
        }
    }
} 