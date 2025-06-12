use core_affinity::CoreId;
use crate::prozessor::ProzessorSpecs;
use std::collections::HashSet;

/*
    Auswahl fürs CPU Pinning nach Reihenfolge:
    1. Den logischen Kern mit der niedrigsten id des physischen Kerns der Hyperthreading hat
    2. den logischen Kern eines pyhsischen Kerns ohne Hyperthreading
    3. den übrigen logischen Kern eines der pyhsisch hyperthreading hat
*/
pub fn reihenfolge(anzahl: usize, prozessor: &ProzessorSpecs) -> Vec<CoreId> {
    
    let mut liste: Vec<CoreId> = Vec::with_capacity(anzahl);

    let mut gesehen: HashSet<u32> = HashSet::with_capacity(anzahl);

    // physische Kerne die hyperthreading haben hinzufügen ohne es zu nutzen
    for a in 0..prozessor.mit_hyperthreading {
        if liste.len() == anzahl {
            break;
        }

        let kern: u32  = a * prozessor.hyperthreads_pro_kern;
       
        // zu Hashmap und Ergebnisliste hinzufügen
        if gesehen.insert(kern) {
            liste.push(CoreId{ id: kern as usize});  
        }    
    }

    let offset = prozessor.mit_hyperthreading * prozessor.hyperthreads_pro_kern; 

    // physische Kerne ohne hyperthreading hinzufügen
    for b in offset..prozessor.logische_kerne {
        if liste.len() == anzahl {
            break;
        }

        // zu Hashmap und Ergebnisliste hinzufügen
        if gesehen.insert(b) {
            liste.push(CoreId{id: b as usize });
        }
    }

    // alle übrigen Hyperthreading hinzufügen
    for c in 0..prozessor.logische_kerne {
        if liste.len() == anzahl {
            break;
        }

        if gesehen.insert(c) {
            liste.push(CoreId{ id: c as usize});
        }
    }

    liste
}