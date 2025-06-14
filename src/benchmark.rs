use crate::{Einstellungen, matrix, pinning, prozessor::ProzessorSpecs};
use crate::algorithmus::{single_thread, manuelle_threads, loop_unrolling, block_tiling, rayon_nutzen, simd_nutzen,
    crossbeam_nutzen};
use std::{process, time::Instant, path::Path, fs::OpenOptions, io::Write};
use core_affinity::{CoreId, set_for_current};
use rayon::{ThreadPoolBuilder, ThreadPool};

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

    let mut gemessen: Vec<BenchmarkEintrag> = Vec::with_capacity((prozessor.logische_kerne - 1) as usize * n.len());

    // Benchmark durchführen
    for i in 2..=prozessor.logische_kerne as usize {

        // Für debug Ausgabe
        let mut ok: bool = true;

        // Reihenfolge für Pinning
        let pinnen: Vec<CoreId> = pinning::reihenfolge(i, &prozessor);

        match eingabe.modus {
            0 => {  println!("Benchmark mit {} Threads. Pinning auf Thread 0", i); }
            1 | 2 | 3 | 4 | 5 | 6 => {  
                    let ids: Vec<usize> = pinnen.iter().map(|kern: &CoreId| kern.id).collect();
                    println!("Benchmark mit {} Threads. Pinning: {:?}", i, ids); }
            _ => { } // Fall nicht möglich da die Eingabe den Wert von Modus prüft
        }


        // Benchmark für jeden Thread mit allen Matrixgrößen durchführen
        for j in 0..n.len() {
            let aktuell: usize = n[j] as usize;

            // Matrix mit Zufallswerten füllen
            let a: Vec<Vec<u32>> = matrix::zufallswerte(aktuell);
            let b: Vec<Vec<u32>> = matrix::zufallswerte(aktuell);

            // leere Ergebnismatrizen
            let mut c: Vec<Vec<u32>> = vec![vec![0; aktuell]; aktuell];
            let mut c_kontrolle: Vec<Vec<u32>> = vec![vec![0; aktuell]; aktuell];

            if eingabe.debug {
                single_thread::starten(&a, &b, &mut c_kontrolle, aktuell, &pinnen[0]);
            }

            // für Rayon da es nicht fair wäre dies zu Zeitmessung hinzuzufügen
            let kopie: Vec<CoreId> = pinnen.clone();

            let start: Instant = Instant::now() ;

            match eingabe.modus {
                    0 => {  if i == 2 { single_thread::starten(&a, &b, &mut c, aktuell, &pinnen[0]);}}
                    1 => { manuelle_threads::starten(&a, &b, &mut c, aktuell, i, &pinnen);}
                    2 => { loop_unrolling::starten(&a, &b, &mut c, aktuell, i, &pinnen);}
                    3 => { block_tiling::starten(&a, &b, &mut c, aktuell, i ,&pinnen);}
                    4 => {
                        // Threadpool erstellen 
                        let pool: ThreadPool = ThreadPoolBuilder::new().num_threads(i)
                                    .start_handler(move |id| { set_for_current(kopie[id]);})
                                    .build().unwrap_or_else(|f| {
                                        println!("\nFehler beim erstellen des Threadpools: {}", f);
                                        process::exit(1);});
                        // Matrixmultiplikation ausführen
                        pool.install(|| { rayon_nutzen::starten(&a, &b, &mut c, aktuell);});}
                    5 => { crossbeam_nutzen::starten(&a, &b, &mut c, aktuell, i, &pinnen); }
                    6 => { simd_nutzen::starten(&a, &b, &mut c, aktuell, i, &pinnen); }
                    _ => { } // nicht möglich da Prüfung des Modus bei Eingabe
                }
           
            // Laufzeit in Millisekunden
            let dauer: f64 = start.elapsed().as_secs_f64() * 1000.0;

            // Ergebnis speichern
            let ergebnis: BenchmarkEintrag = BenchmarkEintrag {n: aktuell as u32, threads: i as u32, laufzeit: dauer};
            gemessen.push(ergebnis);

            if eingabe.debug {
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
    writeln!(datei, "Name: {}, logische Kerne: {}, physische Kerne: {}, hyperthreading: {}", prozessor.name, prozessor.logische_kerne, 
        prozessor.physische_kerne, prozessor.hyperthreads_pro_kern).unwrap();

    for a in gemessen {
        writeln!(datei, "{},{},{}", a.threads, a.n, a.laufzeit).unwrap_or_else(|f| {
            println!("\nFehler beim schreiben der Ergebnisse: {}\n", f);
            process::exit(1);
        });
    }
}