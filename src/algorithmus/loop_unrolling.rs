use std::thread;
use core_affinity::{CoreId, set_for_current};

pub fn starten(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, threads: usize, pinnen: &Vec<CoreId>) {

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

            let (bearbeiten, restliche_zeilen): (&mut[Vec<u32>], &mut[Vec<u32>]) = übrig.split_at_mut(zeilen);
            let anfang: usize = offset;

            let kern: CoreId = pinnen[z];

            s.spawn(move || {
                // Thread pinnen
                set_for_current(kern);

                for z in 0..zeilen {

                    // Zugriff auf aktuelle Zeile zur Bearbeitung
                    let ausgabe: &mut Vec<u32> = &mut bearbeiten[z];

                    let i: usize = anfang + z;

                    for j in 0..n {
                        let mut summe: u32 = 0;
                        
                        // Loop unrolling mit Faktor 4
                        let unroll = n % 4;
                        let grenze = n - unroll;

                        for k in (0..grenze).step_by(4) {
                            summe = summe + a[i][k] * b[k][j] + a[i][k + 1] * b[k + 1][j] +
                                        a[i][k + 2] * b[k + 2][j] + a[i][k + 3] * b[k + 3][j]; 
                        }

                        // restliche Zeilen
                        for k in grenze..n {
                            summe = summe + a[i][k] * b[k][j];
                        }
                        ausgabe[j] = summe;
                    }
                }
            });

            // Updaten für nächsten Thread
            übrig = restliche_zeilen;
            offset = offset + zeilen;
        }
    });
}