use rand::random_range;

pub fn zufallswerte(n: usize) -> Vec<Vec<u32>> {
    let mut matrix: Vec<Vec<u32>> = vec![vec![0; n]; n];
    for i in 0..n {
        for j in 0..n {
            // random_range erzeugt automatisch einmalig einen Seed pro Thread
            // Da nur main die Funktion aufruft, wird nur ein Seed erzeugt
            matrix[i][j] = random_range(0..10);
        }
    }
    matrix
}

/*
    single threaded Matrixmultiplikation zur Kontrolle
*/
pub fn single(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, c: &mut Vec<Vec<u32>>, n: usize) {
    for i in 0..n {
        for j in 0..n {
            let mut summe: u32 = 0;
            for k in 0..n {
                summe = summe + a[i][k] * b[k][j];
            }
            c[i][j] = summe;
        }
    }
} 

/*
    Vergleich ob zwei Matrizen identisch sind
*/
pub fn vergleich(a: &Vec<Vec<u32>>, b: &Vec<Vec<u32>>, n: usize) -> bool {
    for i in 0..n {
        for j in 0..n {
            if a[i][j] != b[i][j] {
                return false;
            }
        }
    }
    true
}