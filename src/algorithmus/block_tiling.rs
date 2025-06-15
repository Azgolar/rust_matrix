use std::thread;
use core_affinity::{set_for_current, CoreId};

pub fn starten(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, threads: usize, pinnen: &Vec<CoreId>) {
    thread::scope(|s|{
        let mut übrig:&mut [Vec<u32>]  = c.as_mut_slice();
        let mut offset: usize = 0;

        // Zeilen pro Thread
        let basis: usize = n / threads;
        let rest: usize = n % threads;

        for z in 0..threads {
            // die ersten Threads bekommen eine zusätzliche Zeile
            let zeilen: usize;
            if z < rest {
                zeilen = basis + 1;
            } 
            else {
                zeilen = basis;
            }

            let (bearbeiten, restliche_zeilen): (&mut[Vec<u32>], &mut[Vec<u32>]) = übrig.split_at_mut(zeilen);
            let anfang: usize = offset;

            let kern: CoreId = pinnen[z];

            s.spawn(move || {
               set_for_current(kern);

            // Ausgabezeile mit null initialisieren
            for i in 0..zeilen {
                for j in 0..n {
                    bearbeiten[i][j] = 0;
                }
            }

            // Block Tiling durchführen
            let block = 8;
            for j_block in (0..n).step_by(block) {
                for k_block in (0..n).step_by(block) {
                    
                    // Zeilen bearbeiten
                    for t in 0..zeilen {
                        let i: usize = anfang + t;
                        let ausgabe: &mut Vec<u32> = &mut bearbeiten[t];

                        // Block bearbeiten
                        for j in j_block..(j_block + block).min(n) {
                            let mut summe: u32 = 0;
                            for k in k_block..(k_block + block).min(n) {
                                summe = summe + a[i][k] * b[k][j];
                            }
                        ausgabe[j] = ausgabe[j] + summe;
                        }
                    }
                }
            }
            });

            // 
            übrig = restliche_zeilen;
            offset = offset + zeilen;
        }
    });
}