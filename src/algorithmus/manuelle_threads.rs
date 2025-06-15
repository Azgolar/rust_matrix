use std::{thread, sync::atomic::{AtomicPtr, AtomicUsize, Ordering}};
use core_affinity::{CoreId, set_for_current};

pub fn starten(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, threads: usize, pinnen: &Vec<CoreId>) {
    
    // Anz
    let zeilen: usize = 4;
    let zähler: AtomicUsize = AtomicUsize::new(0);
    let c_zeiger: AtomicPtr<Vec<u32>> = AtomicPtr::new(c.as_mut_ptr());

    thread::scope(|s|{
        for z in 0..threads {
            let kern: CoreId = pinnen[z];
            let zähler_neu: &AtomicUsize = &zähler;
            let c_neu: &AtomicPtr<Vec<u32>> = &c_zeiger;

            s.spawn(move || {
                set_for_current(kern);

                loop { 
                    let anfang: usize = zähler_neu.fetch_add(zeilen, Ordering::Relaxed);
                    if anfang >= n {
                        break;
                    }

                    let ende: usize = (anfang + zeilen).min(n);

                    let zeiger: *mut Vec<u32> = c_neu.load(Ordering::Relaxed);
                    
                    for i in anfang..ende {
                        let ergebnis: &mut Vec<u32> = unsafe { &mut *zeiger.add(i) };

                        for j in 0..n {
                            let mut summe = 0;
                            for k in 0..n {
                                summe = summe + a[i][k] * b[k][j];
                            }
                            ergebnis[j] = summe;
                        }
                    }
                }
            });
        }
    });
}