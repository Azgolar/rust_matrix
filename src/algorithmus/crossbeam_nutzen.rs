use crossbeam::deque::{Injector, Worker};
use std::thread;
use core_affinity::{CoreId, set_for_current};
use std::sync::Arc;

// Task-Struktur für eine Teilberechnung
struct MatrixTask {
    zeilen_start: usize,
    zeilen_ende: usize,
}

pub fn starten(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize, threads: usize, pinnen: &Vec<CoreId>) {
    // Globale Queue für Tasks
    let injector = Arc::new(Injector::<MatrixTask>::new());
   
    // Worker-Queues für jeden Thread
    let mut workers = Vec::with_capacity(threads);
    let mut stealers = Vec::with_capacity(threads);
   
    for _ in 0..threads {
        let w = Worker::<MatrixTask>::new_fifo();
        stealers.push(w.stealer());
        workers.push(w);
    }
   
    // Tasks erstellen - mehr Tasks als Threads für besseres Load Balancing
    let tasks_per_thread = 4; // Kann angepasst werden
    let total_tasks = threads * tasks_per_thread;
    let zeilen_per_task = n / total_tasks;
    let rest = n % total_tasks;
   
    let mut current = 0;
    for i in 0..total_tasks {
        let task_zeilen = if i < rest { zeilen_per_task + 1 } else { zeilen_per_task };
        injector.push(MatrixTask {
            zeilen_start: current,
            zeilen_ende: current + task_zeilen,
        });
        current += task_zeilen;
    }
   
    // Convert stealers to Arc for sharing
    let stealers = Arc::new(stealers);
    
    // Threads starten
    thread::scope(|s| {
        // Get raw pointers to the actual data, not the outer Vecs
        let a_ptr = a.as_ptr() as usize;
        let b_ptr = b.as_ptr() as usize;
        let c_ptr = c.as_mut_ptr() as usize;
        
        for thread_id in 0..threads {
            let worker = workers.pop().unwrap();
            let stealers = Arc::clone(&stealers);
            let injector = Arc::clone(&injector);
            let kern = pinnen[thread_id];
           
            s.spawn(move || {
                // Thread pinnen
                set_for_current(kern);
                
                // Reconstruct pointers from usize
                let a_ptr = a_ptr as *const Vec<u32>;
                let b_ptr = b_ptr as *const Vec<u32>;
                let c_ptr = c_ptr as *mut Vec<u32>;
               
                // Lokale Arbeit finden oder stehlen
                loop {
                    // 1. Versuche Task aus lokaler Queue
                    let task = worker.pop().or_else(|| {
                        // 2. Versuche aus globaler Queue
                        injector.steal().success().or_else(|| {
                            // 3. Versuche von anderen Threads zu stehlen
                            stealers.iter()
                                .enumerate()
                                .filter(|(i, _)| *i != thread_id)
                                .map(|(_, s)| s.steal())
                                .find_map(|s| s.success())
                        })
                    });
                   
                    match task {
                        Some(task) => {
                            // Task ausführen
                            unsafe {
                                for i in task.zeilen_start..task.zeilen_ende {
                                    for j in 0..n {
                                        let mut summe: u32 = 0;
                                        for k in 0..n {
                                            // Get pointers to inner vectors and then to elements
                                            let a_row = &*a_ptr.add(i);
                                            let b_row = &*b_ptr.add(k);
                                            let a_elem = a_row.as_ptr().add(k);
                                            let b_elem = b_row.as_ptr().add(j);
                                            summe += *a_elem * *b_elem;
                                        }
                                        // Get pointer to result location
                                        let c_row = &mut *c_ptr.add(i);
                                        let c_elem = c_row.as_mut_ptr().add(j);
                                        *c_elem = summe;
                                    }
                                }
                            }
                        }
                        None => {
                            // Keine Arbeit mehr verfügbar
                            break;
                        }
                    }
                }
            });
        }
    });
}