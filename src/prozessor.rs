use std::fs::read_to_string;

#[derive(Debug)]
pub struct ProzessorSpecs {
    pub name: String,
    pub logisch: u32,
    pub physisch: u32,
    pub hyperthreading: u32
}

/*
    setzt die ProzessorSpezifikationen im Struct
*/
impl ProzessorSpecs {
    pub fn new() -> Self {

        // Default Werte
        let mut daten: ProzessorSpecs = ProzessorSpecs {
            name: String::new(),
            logisch: 0,
            physisch: 0,
            hyperthreading: 0
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
                else if eintrag == "siblings" {
                    daten.logisch = wert.parse().unwrap_or(0);
                }
                else if eintrag == "cpu cores" {
                    daten.physisch = wert.parse().unwrap_or(0);
                }

                if !daten.name.is_empty() && daten.logisch != 0 && daten.physisch != 0 {
                    break;
                }
            }
        }

        if daten.physisch > 0 {
            daten.hyperthreading = daten.logisch / daten.physisch;
        }
        else {
            daten.hyperthreading = 1;
        }

        if daten.name.is_empty() || daten.logisch == 0 || daten.physisch == 0 {
            println!("\nFehler beim Auslesen der Prozessordaten")
        }

        daten
    }
}
