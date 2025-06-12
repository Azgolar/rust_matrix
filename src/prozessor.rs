use std::{fs::read_to_string, process, process::Command, io::{BufReader, BufRead}, collections::HashMap};
use core_affinity::get_core_ids;


#[derive(Debug)]
pub struct ProzessorSpecs {
    pub name: String,                      // Name des Prozessors
    pub logische_kerne: u32,               // Anzahl der logischen Kerne
    pub physische_kerne: u32,              // Anzahl der physischen Kerne
    pub hyperthreads_pro_kern: u32,        // Anzahl der Hyperthreads pro physischem Kern mit Hyperthreads
    pub mit_hyperthreading: u32,           // Anzahl Kerne mit Hyperthreading
    pub ohne_hyperthreading: u32           // Anzahl Kerne ohne Hyperthreading
}

/*
    setzt die ProzessorSpezifikationen im Struct
*/
impl ProzessorSpecs {
    pub fn new() -> Self {

        // Default Werte
        let mut daten: ProzessorSpecs = ProzessorSpecs {
            name: String::new(),
            logische_kerne: 0,
            physische_kerne: 0,
            hyperthreads_pro_kern: 0,
            mit_hyperthreading: 0,
            ohne_hyperthreading: 0
        };

        // cpuinfo ist gelesene Zeilen oder bei Fehler wegen unwrap_or_default ein leerer String
        let cpuinfo: String = read_to_string("/proc/cpuinfo").unwrap_or_default();

        for zeile in cpuinfo.lines() {

            // aufteilen der Zeile in ein Tupel
            if let Some((eintrag, wert)) = zeile.split_once(":") {
                let eintrag: &str = eintrag.trim();
                let wert: &str = wert.trim();            

                if eintrag == "model name" {
                    daten.name = wert.to_string();
                }

                if !daten.name.is_empty() {
                    break;
                }
            }
        }

        (daten.mit_hyperthreading, daten.ohne_hyperthreading) = kernart_bestimmen();

        // Anzahl logischer Kerne

        let anzahl = get_core_ids().unwrap_or_else(|| {
            println!("\nFehler beim Lesen der logischen Kern ids für CPU Pinning\n");
            process::exit(1);    
        }); 
        daten.logische_kerne = anzahl.len() as u32;

        // Anzahl physische Kerne
        daten.physische_kerne = daten.mit_hyperthreading as u32 + daten.ohne_hyperthreading as u32;

        if daten.physische_kerne > 0 {
            daten.hyperthreads_pro_kern = (daten.logische_kerne - daten.ohne_hyperthreading as u32) / (daten.physische_kerne -daten.ohne_hyperthreading as u32) ;
        }
        else {
            daten.hyperthreads_pro_kern = 1;
        }

        if daten.name.is_empty() || daten.logische_kerne == 0 || daten.physische_kerne == 0 || daten.mit_hyperthreading == 0 || daten.ohne_hyperthreading == 0 {
            println!("\nFehler beim Auslesen der Prozessordaten")
        }

        daten
    }
}


/*
    Bestimmt die Kern ids mit Hyperthreading und ohne Hyperthreading
*/
fn kernart_bestimmen() -> (u32, u32) {

    // lscpu -p lesen
    let lscpu = Command::new("lscpu").arg("-p").output().unwrap_or_else(|_| {
        println!("\nFehler beim lesen von lscpu\n");
        process::exit(1);
    });

    let mut map: HashMap<u32, u32> = HashMap::new();

    // Zeile für zeile lesen
    let lesen = BufReader::new(&lscpu.stdout[..]);
    for a in lesen.lines() {

        // Zeile parsen und Fehler überspringen
        let z = match a {
            Ok(b) => b,
            Err(_) => continue
        };

        let zeile = z.trim();

        // Kommentare überspringen
        if zeile.starts_with('#') || zeile.is_empty() {
            continue;
        }

        // Es muss mindstens zwei Spalten geben
        let spalten: Vec<&str> = zeile.split(',').collect();
        if spalten.len() < 2 {
            continue;
        }

        let kern = spalten[1].parse::<u32>().unwrap();

        if map.contains_key(&kern) {
            let zahl = map.get_mut(&kern).unwrap();
            *zahl = *zahl + 1;
        }
        else {
            map.insert(kern, 1);
        }
    }

    // Aufteilen in Kerne mit Hyperthreading und ohne Hyperthreading
    let mut mit_hyperthreading: u32 = 0;
    let mut ohne_hyperthreading: u32 = 0;

    for &anzahl in map.values() {
        if anzahl > 1 {
            mit_hyperthreading = mit_hyperthreading + 1;
        }
        else {
            ohne_hyperthreading = ohne_hyperthreading + 1;
        }
    }

    (mit_hyperthreading, ohne_hyperthreading)
} 