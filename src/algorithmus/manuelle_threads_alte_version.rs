use std::{sync::Arc, thread::JoinHandle, thread::spawn};
use core_affinity::{CoreId, set_for_current};

pub fn manuell(a: Arc<Vec<Vec<u32>>>, b: Arc<Vec<Vec<u32>>>, c: &mut Vec<Vec<u32>>, n:usize, threads: usize, pinnen: &Vec<CoreId>) {
    
    // Zeilen pro Thread
    let basis: usize = n / threads;
    let rest: usize = n % threads;

    // Join für Threads
    let mut handle: Vec<JoinHandle<Vec<(usize, Vec<u32>)>>> = Vec::with_capacity(threads);

    for z in 0..threads {
        
        // jeder Thread benötigt seinen eigenen Arc Zeiger
        let a_arc: Arc<Vec<Vec<u32>>> = Arc::clone(&a);
        let b_arc: Arc<Vec<Vec<u32>>> = Arc::clone(&b);

        let kern: CoreId = pinnen[z];

        // Zeilenbereich für jeden Thread berechnen
        let zeilen: usize;
        let offset: usize;
        if z < rest {
            zeilen = basis + 1;
            offset = z;
        }
        else {
            zeilen = basis;
            offset = rest;
        }
        let anfang: usize = z * basis + offset;
        let ende: usize = anfang + zeilen;

        let erzeugen = spawn(move || {
            
            // auf logischen Kern pinnen
            set_for_current(kern);
        
            let mut zwischenergebnis: Vec<(usize, Vec<u32>)> = Vec::with_capacity(ende - anfang);
            for i in anfang..ende {
                let mut temporär = Vec::with_capacity(n);
                for j in 0..n {
                    let mut summe: u32 = 0;
                    for k in 0..n {
                        summe = summe + a_arc[i][k] * b_arc[k][j];
                    }
                    temporär.push(summe);
                }
                zwischenergebnis.push((i, temporär));
            }
        // Tupel (Zeilenindex, Zeile) von Thread zurückgeben
        zwischenergebnis
        });
        // Thread handle für Join speichern
        handle.push(erzeugen);
    }

    // Ergebnismatrix zusammenbauen 
    // Es werden nur Zeiger geändert und keine Daten kopiert -> O(1)
    for y in handle {
        for (i, row) in y.join().unwrap() {
            c[i] = row;
        }
    }
}