use std::{thread, simd::u32x8, simd::Simd};
use core_affinity::{CoreId, set_for_current};

/*
    nutzt Simd und loop unrolling
    Zum Testen wurde ein i7-14700k verwendet. Der Prozessor hat AVX2 Register mit Breite 256 bit (8 * 32 bit)
*/
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

                    // Zeilenindex
                    let i: usize = anfang + z;

                    let blöcke: usize = (n / 8) * 8;

                    // iterieren über Spalten (=Simd Blöcke)
                    for j in (0..blöcke).step_by(8) {
                        let mut summe: Simd<u32, 8> = u32x8::splat(0);

                        // Summe
                        for k in 0..n {
                            // auf alle 8 Lanes übertragen
                            let teil1: Simd<u32, 8> = u32x8::splat(a[i][k]);

                            let teil2: Simd<u32, 8> = u32x8::from_array([b[k][j], b[k][j+1], b[k][j+2],
                                b[k][j+3], b[k][j+4], b[k][j+5], b[k][j+6], b[k][j+7]]);
                            summe = summe + teil1 * teil2;
                        }
                    
                        // Speichern der Zwishchenergebnisse
                        let zwischen: [u32; 8] = summe.to_array();
                        for l in 0..8 {
                            ausgabe[j + l] = zwischen[l];
                        }
                    }

                    // übrige Spalten (nicht durch 8 teilbar)
                    for l in blöcke..n {
                        let mut summe2 = 0;
                        for m in 0..n {
                            summe2 = summe2 + a[i][m] * b[m][l];
                        }
                        ausgabe[l] = summe2;
                    }
                }
            });

            // Updaten für den nächsten Thread
            übrig = restliche_zeilen;
            offset = offset + zeilen;
        }
    });

}