use std::thread;
use core_affinity::{CoreId, set_for_current};

pub fn tiling(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, threads: usize, pinnen: &Vec<CoreId>) {

    thread::scope(|s| {
        let mut übrig: &mut [Vec<u32>] = c.as_mut_slice();
        let mut offset: usize = 0;

        // Zeilen pro Thread
        let basis: usize = n / threads;
        let rest: usize = n % threads;

        for z in 0..threads {
            // die ersten Threads bekommen eine zusätzlich Zeile
            let zeilen: usize;
            if z < rest {
                zeilen = basis + 1;
            } 
            else {
                zeilen = basis;
            }

            let (bearbeiten, restliche_zeilen) = übrig.split_at_mut(zeilen);
            let anfang: usize = offset;

            let kern: CoreId = pinnen[z];

            s.spawn(move || {
                // Thread pinnen
                set_for_current(kern);

                let ende = anfang + zeilen;

                let block = 64;
                for ii in (0..n).step_by(block) {
                    let block_i_anfang = ii;
                    let bloock_i_ende = (ii + block).min(n);

                    if block_i_ende <= anfang || block_i_start >= ende {
                        continue;
                    }

                    for jj in (0..n).step_by(block) {
                        let block_k_start = kk;
                        let block_k_end = (kk + block).min(n);

                        let i0 = block_i_start.max(anzahl);
                        let i1 = block_i_end.min.(ende);

                        let machen = &mut bearneiten[i - anfang];
                        for j in block_j_start..block_j_ende {
                            let mut summe = machen[j];
                            for k in block_k_afnang..block_k_ende {
                                summe = summe + a[i][k] * b[k][j];
                            }
                            machen[j] = summe;
                        }
                    }
                }
            });
            // Updaten für nächsten Thread
            übrig = restliche_zeilen;
            offset = offset + zeilen;
        }
    });

}