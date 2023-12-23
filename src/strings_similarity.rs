use std::collections::HashMap;

// Taken from: https://github.com/DimaKudosh/difflib/blob/master/src/sequencematcher.rs
pub fn get_common_substring(str1: &str, str2: &str) -> String {
    let first_sequence: Vec<char> = str1.chars().collect();
    let second_sequence: Vec<char> = str2.chars().collect();

    let mut best_i: usize = 0;
    let mut best_j: usize = 0;
    let mut best_size: usize = 0;

    let mut second_sequence_elements = HashMap::new();
    for (i, item) in second_sequence.iter().enumerate() {
        let counter = second_sequence_elements
            .entry(item)
            .or_insert_with(Vec::new);
        counter.push(i);
    }

    let mut j2len: HashMap<usize, usize> = HashMap::new();
    for (i, item) in first_sequence.iter().enumerate() {
        let mut new_j2len: HashMap<usize, usize> = HashMap::new();
        if let Some(indexes) = second_sequence_elements.get(item) {
            for j in indexes {
                let j = *j;
                let mut size = 0;
                if j > 0 {
                    if let Some(k) = j2len.get(&(j - 1)) {
                        size = *k;
                    }
                }
                size += 1;
                new_j2len.insert(j, size);
                if size > best_size {
                    best_i = i + 1 - size;
                    best_j = j + 1 - size;
                    best_size = size;
                }
            }
        }
        j2len = new_j2len;
    }

    for _ in 0..2 {
        while best_i > 0
            && best_j > 0
            && first_sequence.get(best_i - 1) == second_sequence.get(best_j - 1)
        {
            best_i -= 1;
            best_j -= 1;
            best_size += 1;
        }
        while best_i + best_size < first_sequence.len()
            && best_j + best_size < second_sequence.len()
            && first_sequence.get(best_i + best_size) == second_sequence.get(best_j + best_size)
        {
            best_size += 1;
        }
    }

    let res = String::from_iter(&first_sequence[best_i..(best_i + best_size)]);
    res
}
