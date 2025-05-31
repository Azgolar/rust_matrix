use core_affinity::{CoreId, get_core_ids};
use std::process;
use crate::prozessor::ProzessorSpecs;

/*
    Bestimmt eine Liste von CPU Kernen auf die Threads gepinnt werden sollen.

    Vorgehen: 
    1. zuerst wird auf physische Kerne gepinnt
    2. falls keine physischen Kerne mehr übrig sind wird auf logische Kerne gepinnt 
    
    Die Liste der Kern IDs ist in aufsteigender Reihenfolge
*/
pub fn reihenfolge(anzahl: usize, prozessor: &ProzessorSpecs) -> Vec<CoreId> {
    // Alle Kerne Ids lesen
    let ids: Vec<CoreId> = get_core_ids().unwrap_or_else(|| {
        println!("\nFehler beim Lesen der Kern-IDs für CPU Pinning\n");
        process::exit(1);    
    }); 

    let mut reihenfolge: Vec<CoreId> = Vec::with_capacity(anzahl);

    // physische Kerne zuerst hinzufügen
    for i in 0..prozessor.physisch as usize {
        if reihenfolge.len() >= anzahl {
            break;
        }

        let index: usize = i * prozessor.hyperthreading as usize;
        let gefunden: Option<&CoreId> = ids.iter().find(|c: &&CoreId| c.id == index);
        reihenfolge.push(*gefunden.unwrap());
    }

    // restlichen logischen Kerne hinzufügen
    for j in ids.iter() {
        if reihenfolge.len() >= anzahl {
            break;
        }
        // nur hinzufügen falls noch nicht in Liste
        if !reihenfolge.iter().any(|c: &CoreId| c.id == j.id) {
            reihenfolge.push(*j)
        }
    }

    reihenfolge
}
    
