// adopted from https://github.com/TheAlgorithms/Rust/blob/master/src/dynamic_programming/knapsack.rs

/// knapsack_table(w, weights) returns the knapsack table (`n`, `m`) with maximum values, where `n` is number of items
///
/// # Arguments:
///   * `w` - knapsack capacity
///   * `weights` - set of weights for each item
fn knapsack_table(w: &usize, weights: &[usize]) -> Vec<Vec<usize>> {
    // Initialize `n` - number of items
    let n: usize = weights.len();
    // Initialize `m`
    // m[i, w] - the maximum value that can be attained with weight less that or equal to `w` using items up to `i`
    let mut m: Vec<Vec<usize>> = vec![vec![0; w + 1]; n + 1];

    for i in 0..=n {
        for j in 0..=*w {
            // m[i, j] compiled according to the following rule:
            if i == 0 || j == 0 {
                m[i][j] = 0;
            } else if weights[i - 1] <= j {
                // If `i` is in the knapsack
                // Then m[i, j] is equal to the maximum value of the knapsack,
                // where the weight `j` is reduced by the weight of the `i-th` item and the set of admissible items plus the value `k`
                m[i][j] = std::cmp::max(weights[i - 1] + m[i - 1][j - weights[i - 1]], m[i - 1][j]);
            } else {
                // If the item `i` did not get into the knapsack
                // Then m[i, j] is equal to the maximum cost of a knapsack with the same capacity and a set of admissible items
                m[i][j] = m[i - 1][j]
            }
        }
    }

    m
}

/// knapsack_items(weights, m, i, j) returns the indices of the items of the optimal knapsack (from 1 to `n`)
///
/// # Arguments:
///   * `weights` - set of weights for each item
///   * `m` - knapsack table with maximum values
///   * `i` - include items 1 through `i` in knapsack (for the initial value, use `n`)
///   * `j` - maximum weight of the knapsack
fn knapsack_items(weights: &[usize], m: &[Vec<usize>], i: usize, j: usize) -> Vec<usize> {
    if i == 0 {
        return vec![];
    }
    if m[i][j] > m[i - 1][j] {
        let mut knap: Vec<usize> = knapsack_items(weights, m, i - 1, j - weights[i - 1]);
        knap.push(i - 1);
        knap
    } else {
        knapsack_items(weights, m, i - 1, j)
    }
}

/// knapsack(w, weights, values) returns the tuple where first value is "optimal profit",
/// and the last value is "indices of items", that we got (from 1 to `n`)
///
/// # Arguments:
///   * `w` - knapsack capacity
///   * `weights` - set of weights for each item
///
/// # Complexity
///   - time complexity: O(nw),
///   - space complexity: O(nw),
///
/// where `n` and `w` are "number of items" and "knapsack capacity"
pub fn knapsack(w: usize, weights: Vec<usize>) -> (usize, Vec<usize>) {
    // Initialize `n` - number of items
    let n: usize = weights.len();
    // Find the knapsack table
    let m: Vec<Vec<usize>> = knapsack_table(&w, &weights);
    println!("{:?}", m);
    // Find the indices of the items
    let items: Vec<usize> = knapsack_items(&weights, &m, n, w);
    // Return result
    (m[n][w], items)
}
