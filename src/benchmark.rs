use crate::Einstellungen;
use crate::matrix;
use crate::algorithmus::multi_threads;
use crate::prozessor::ProzessorSpecs;
use crate::pinning;
use std::{process, time::Instant, path::Path, fs::OpenOptions, io::Write};
use core_affinity::{CoreId};

struct BenchmarkEintrag {
    n: u32,
    threads: u32,
    laufzeit: f64
}


pub fn beginnen(eingabe: &Einstellungen, n: Vec<u32>) {

    let prozessor: ProzessorSpecs = ProzessorSpecs::new();

    if eingabe.debug {
        println!("{:?}\n", prozessor);
    }

    let mut gemessen: Vec<BenchmarkEintrag> = Vec::with_capacity((prozessor.logisch - 1) as usize * n.len());

    // Benchmark durchführen
    for i in 2..=prozessor.logisch as usize {

        // Für debug Ausgabe
        let mut ok: bool = true;

        // Reihenfolge für Pinning
        let pinnen: Vec<CoreId> = pinning::reihenfolge(i, &prozessor);

        let ids: Vec<usize> = pinnen.iter().map(|kern: &CoreId| kern.id).collect();
        println!("Benchmark mit {} Threads. Pinning: {:?}", i, ids);

        // Benchmark für jeden Thread mit allen Matrixgrößen durchführen
        for j in 0..n.len() {
            let aktuell: usize = n[j] as usize;

            // Matrix mit Zufallswerten füllen
            let a: Vec<Vec<u32>> = matrix::zufallswerte(aktuell);
            let b: Vec<Vec<u32>> = matrix::zufallswerte(aktuell);

            // leere Ergebnismatrix
            let mut c: Vec<Vec<u32>> = vec![vec![0; aktuell]; aktuell];

            let start: Instant = Instant::now();

            starten(&a, &b, &mut c, i, aktuell, &pinnen, eingabe.modus);
            
            // Laufzeit in Millisekunden
            let dauer: f64 = start.elapsed().as_secs_f64() * 1000.0;

            // Ergebnis speichern
            let ergebnis: BenchmarkEintrag = BenchmarkEintrag {n: aktuell as u32, threads: i as u32, laufzeit: dauer};
            gemessen.push(ergebnis);

            if eingabe.debug {
                let mut c_kontrolle: Vec<Vec<u32>> = vec![vec![0; aktuell]; aktuell];
                starten(&a, &b, &mut c_kontrolle, i, aktuell, &pinnen, 0);
                let z: bool  = matrix::vergleich(&c_kontrolle, &c, aktuell);
                if !z {
                    ok = false;
                    println!("Ergebnis falsch bei threads = {}, n = {}", i, aktuell);
                }
            }
        }

        if eingabe.debug && ok {
            println!("Ergebnisse sind korrekt");
        }
    }

    // Ergebnisse in Datei speichern
    speichern(&eingabe.name, &prozessor, &gemessen);
}

/*
    Matrixmultiplikation durchführen
*/
fn starten(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, i: usize, aktuell: usize, pinnen: &Vec<CoreId>, modus: u32) {

    if modus == 0 {
        matrix::single(&a, &b, c, aktuell);
    }
    else if modus == 1 {
        multi_threads::manuelle_threads(&a, &b, c, aktuell, i, &pinnen);
    }
}



fn speichern(name: &str, prozessor: &ProzessorSpecs, gemessen: &Vec<BenchmarkEintrag>) {
    let pfad = Path::new(name);
    if pfad.exists() {
        println!("\nHinweis: {} wurde überschrieben\n", name);
    }

    let mut datei:std::fs::File = OpenOptions::new().write(true).create(true).truncate(true)
        .open(pfad).unwrap_or_else(|f| { println!("\nFehler beim öffnen der Datei {}: {}", name, f);
        process::exit(1);
    });

    // Kopfzeile hinzufügen
    writeln!(datei, "Name: {}, logisch: {}, physisch: {}, hyperthreading: {}", prozessor.name, prozessor.logisch, 
        prozessor.physisch, prozessor.hyperthreads_pro_kern).unwrap();

    for a in gemessen {
        writeln!(datei, "{},{},{}", a.threads, a.n, a.laufzeit).unwrap_or_else(|f| {
            println!("\nFehler beim schreiben der Ergebnisse: {}\n", f);
            process::exit(1);
        });
    }
}