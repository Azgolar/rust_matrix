use crate::{Einstellungen, matrix, pinning, prozessor::ProzessorSpecs};
use crate::algorithmus::{single_thread, manuelle_threads_neue_version, manuelle_threads_alte_version, loop_unrolling,
    block_tiling, rayon_nutzen};
use std::{process, time::Instant, path::Path, fs::OpenOptions, io::Write, sync::Arc};
use core_affinity::CoreId;
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
            1 | 2 | 3 | 4 => {  
                    let ids: Vec<usize> = pinnen.iter().map(|kern: &CoreId| kern.id).collect();
                    println!("Benchmark mit {} Threads. Pinning: {:?}", i, ids); }
            5 =>  { println!("Benchmark mit {} Threads", i); } 
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
                single_thread::single(&a, &b, &mut c_kontrolle, aktuell, &pinnen[0]);
            }

            let start: Instant = Instant::now();

            match eingabe.modus {
                    0 => {  single_thread::single(&a, &b, &mut c, aktuell, &pinnen[0]); }
                    1 => {  let a_arc: Arc<Vec<Vec<u32>>> = Arc::new(a);
                            let b_arc: Arc<Vec<Vec<u32>>> = Arc::new(b);                          
                            manuelle_threads_alte_version::manuell(Arc::clone(&a_arc), Arc::clone(&b_arc), &mut c, aktuell, i, &pinnen); }
                    2 => {  manuelle_threads_neue_version::manuell(&a, &b, &mut c, aktuell, i, &pinnen); }
                    3 => {  loop_unrolling::unroll(&a, &b, &mut c, aktuell, i, &pinnen); }
                    4 => {  block_tiling::tiling(&a, &b, &mut c, aktuell, i, &pinnen); } 
                    5 => {  let pool: ThreadPool = ThreadPoolBuilder::new().num_threads(i).build().unwrap_or_else(|f| {
                            println!("\nWarnung: Fehler beim erstellen des Threadpools: {}", f);
                            process::exit(1)});
                            pool.install( || { rayon_nutzen::parallel(&a, &b, &mut c, aktuell)}); }
                    _ => { } // Fall nicht möglich da die Eingabe die Korrektheit von Modus prüft
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