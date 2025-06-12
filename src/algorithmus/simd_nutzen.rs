use std::{thread, simd::u32x8, simd::Simd};
use core_affinity::{CoreId, set_for_current};

/*
    nutzt Simd und loop unrolling
    Zum Testen wurde ein i7-14700k verwendet. Der Prozessor hat AVX2 Register mit Breite 256 bit (8 * 32 bit)
*/
pub fn optimiert(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, threads: usize, pinnen: &Vec<CoreId>) {
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

                    for j in (0..n).step_by(8) {
                        // Es bleibt ein Rest
                        if j + 8 > n {
                            break
                        }

                        let mut summe: Simd<u32, 8> = u32x8::splat(0);

                        for k in (0..n).step_by(4) {
                            if k + 4 > n {
                                for uebrig in k..n {
                                    let zwischen0: Simd<u32, 8> = u32x8::splat(a[i][uebrig]);
                                    let zwischen1: Simd<u32, 8> = u32x8::from_array([b[uebrig][j], b[uebrig][j+1], 
                                    b[uebrig][j+2], b[uebrig][j+3], b[uebrig][j+4], b[uebrig][j+5],
                                    b[uebrig][j+6], b[uebrig][j+7]]);

                                    summe = summe + zwischen0 * zwischen1;
                                }
                                break;
                            }

                            // ersten vier Iterationen
                            let zwischen2: Simd<u32, 8> = u32x8::splat(a[i][k]);
                            let zwischen3: Simd<u32, 8> = u32x8::from_array([b[k][j], b[k][j+1], 
                                b[k][j+2], b[k][j+3], b[k][j+4], b[k][j+5], b[k][j+6], b[k][j+7]]);

                            summe = summe + zwischen2 * zwischen3;

                            // zweiten vier Iterationen
                            let zwischen4: Simd<u32, 8> = u32x8::splat(a[i][k+1]);
                            let zwischen5: Simd<u32, 8> = u32x8::from_array([b[k+1][j], b[k+1][j+1], 
                                b[k+1][j+2], b[k+1][j+3], b[k+1][j+4], b[k+1][j+5], b[k+1][j+6], b[k+1][j+7]]);

                            summe = summe + zwischen4 * zwischen5;

                            // dritten vier Iterationen
                            let zwischen6: Simd<u32, 8> = u32x8::splat(a[i][k+2]);
                            let zwischen7: Simd<u32, 8> = u32x8::from_array([b[k+2][j], b[k+2][j+1], 
                                b[k+2][j+2], b[k+2][j+3], b[k+2][j+4], b[k+2][j+5], b[k+2][j+6], b[k+2][j+7]]);

                            summe = summe + zwischen6 * zwischen7;

                            // vierten vier Iterationen
                            let zwischen8: Simd<u32, 8> = u32x8::splat(a[i][k+3]);
                            let zwischen9: Simd<u32, 8> = u32x8::from_array([b[k+3][j], b[k+3][j+1], 
                                b[k+3][j+2], b[k+3][j+3], b[k+3][j+4], b[k+3][j+5], b[k+3][j+6], b[k+3][j+7]]);

                            summe = summe + zwischen8 * zwischen9;
                        }

                        // rest
                        let noch = (n / 8) * 8; 
                        for i in noch..n {
                            let mut summe = 0;
                            for j in 0..n {
                                summe = summe + a[i][j] * b[i][j];
                            }
                            ausgabe[j] = summe;
                        }
                    }

                }
            });

            // Updaten für den nächsten Thread
            übrig = restliche_zeilen;
            offset = offset + zeilen;
        }
    });

}