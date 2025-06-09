use core_affinity::{set_for_current, CoreId};
use std::{mem::transmute, thread};


pub fn unsicher(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, threads: usize, pinnen: &Vec<CoreId>) {

    // die ersten Threads bekommen eine zusätzlich Zeile
    let basis = n / threads;
    let rest = n % threads;
    let mut offset = 0;

    /*
        Umwandeln in eine statische Lebenszeit. Dies ist unsicher da wir sagen dass a und b für die gesamte 
        Programmlaufzeit gültig ist aber es nur bis zum Ende des Funktionsaufrufs garantiert ist
    */
    let a_neu: &'static Vec<Vec<u32>> = unsafe { transmute(a) };
    let b_neu: &'static Vec<Vec<u32>> = unsafe { transmute(b) };
    // Der Zeiger von c wird in einen usize Wert umgewandelt sodass mehrere Threads auf c zugreifen können
    let c_neu = c as *mut Vec<Vec<u32>> as usize;

    thread::scope(|s| {
        let mut zeilen: usize;
        for z in 0..threads {
            if z < rest {
                zeilen = basis + 1;
            } 
            else {
                zeilen = basis;
            }

            let start = offset;
            let ende = start + zeilen;
            offset = ende;

            let kern: CoreId = pinnen[z];

            s.spawn(move || {
                set_for_current(kern);

                unsafe {
                    /*
                        Wandelt zuerst den usize Wert zurück in einen Zeiger um und danach in eine
                        Referenz zum 2D Vektor. 

                        --> Es gibt mehrere mut Referenzen auf c existieren. Dies ist nicht erlaubt in Rust
                        ohne unsafe.
                        In der Theorie wären so Datenrennen möglich aber hier ist durch thread:: scope und 
                        den eigenen Zeilenbereich für jeden Thread garantiert dass es keine Konflikte gibt
                    */
                    let c = &mut *(c_neu as *mut Vec<Vec<u32>>);

                    for i in start..ende {
                        for j in 0..n {
                            let mut summe = 0;
                            for k in 0..n {
                                summe = summe + a_neu[i][k] * b_neu[k][j];
                            }
                            c[i][j] = summe;
                        }
                    }
                }
            });
        }
    });
}