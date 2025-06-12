use std::thread;
use core_affinity::{CoreId, set_for_current};

pub fn manuell(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, threads: usize, pinnen: &Vec<CoreId>) {

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

                for z in 0..zeilen {

                    // Zugriff auf aktuelle Zeile zur Bearbeitung
                    let ausgabe: &mut Vec<u32> = &mut bearbeiten[z];

                    // Zeilenindex
                    let i: usize = anfang + z;

                    for j in 0..n {
                        let mut summe: u32 = 0;
                        for k in 0..n {
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