mod benchmark;
mod matrix;
mod algorithmus;
mod pinning;
mod prozessor;
use clap::{Parser, value_parser};

#[derive(Parser)]
#[command(author = "Stefan Brand", version = "1.0", about = "Parallele Matrixmultiplikation", 
    long_about = "Beispiel: ./name -n 5000 -m 1 -a matrix -d:")]
pub struct Einstellungen {
    /// Erzeugt Matrixgrößen von 2 bis n
    #[arg(short = 'n', value_parser = value_parser!(u32).range(2..))]
    n_max: u32,

    /// Modus: 1 = manuelle Threads, 2 = Rayon, 3 = 
    #[arg(short = 'm', value_parser = value_parser!(u32).range(0..=5))]
    modus: u32,

    /// Dateiname zum Speichern der Ergebnisse
    #[arg(short = 'a', value_parser = value_parser!(String), default_value = "ergebnis.txt")]
    name: String,

    /// Debug Ausgabe aktivieren (optional)
    #[arg(short = 'd')]
    debug: bool
}

fn main() {

    // let mut eingabe = Args::parse();
    let test: [&'static str; 8] = ["name", "-n", "20", "-m", "2", "-a", "matrix", "-d"];
    let mut eingabe: Einstellungen = Einstellungen::parse_from(&test);

    // falls nötig .txt an Dateiname hinzufügen
    if !eingabe.name.to_lowercase().ends_with(".txt") {
        eingabe.name.push_str(".txt");
    }

    // Matrixgrößen von 2 bis n erzeugen
    let n: Vec<u32> = konvertieren(2, eingabe.n_max);
    if eingabe.debug {
        let umwandeln: &'static str = match eingabe.modus {
            1 => "single Thread",
            2 => "manuelle Threads",
            3 => "Rayon",
            _ => "Fehler, Modus nicht bekannt"
        };
        println!("\nEinstellungen:\nMatrixgrößen: {:?}\nModus:        {}\nLogdatei:     {}", n, umwandeln, eingabe.name);
    }

    // Benchmark ausführen
    benchmark::beginnen(&eingabe, n);

    
}


/*
    Erzeugt einen Vektor im Bereich [anfang, ende]
    - Die Schrittweite wird adaptiv größer
    - Matrixgrößen mit 2^x Potenzen sind immer enthalten
*/
fn konvertieren(anfang: u32, ende: u32) -> Vec<u32> {
    let mut liste: Vec<u32> = Vec::new();

    liste.push(anfang);

    let mut letztes: u32 = anfang;

    // nächste Zweierpotenz
    let mut zweier: u32 = 4;

    // Schrittweite festlegen
    while letztes < ende {
        let schritt: u32 = match letztes {
            2..=9       => 4,
            10..=99     => 6,
            100..=999   => 100,
            1000..=9999 => 500,
            _           => 1000  
        };

        let aktuell: u32 = letztes + schritt;

        // Falls nötig Zweierpotenz hinzufügen
        if zweier > letztes && zweier <= ende && zweier < aktuell {
            liste.push(zweier);
            zweier = zweier * 2;
        }

        // Prüfen ob obere Grenze überschritten wurde
        if aktuell >= ende {
            // Ende erreicht
            liste.push(ende);
            break;
        }
        else {
            // Ende noch nicht erreicht
            liste.push(aktuell);
            letztes = aktuell;
        }
    }
    liste
}