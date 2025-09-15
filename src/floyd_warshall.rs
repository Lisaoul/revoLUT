use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const NOHOP: i32 = 1832;
#[inline]
fn idx(n: usize, i: usize, j: usize) -> usize { i * n + j }

fn ensure_exists(p: &str) {
    if !Path::new(p).exists() {
        eprintln!(
            "Fichier introuvable: {}\nDossier courant: {}",
            p,
            std::env::current_dir().unwrap().display()
        );
        std::process::exit(1);
    }
}

/// -------- Prétraitement --------
/// Garde les `n_nodes` plus petits IDs de nœuds et écrit seulement les arêtes internes (u v).
fn preprocess_first_n_nodes(
    input_txt: &str,
    output_txt: &str,
    n_nodes: usize,
) -> std::io::Result<()> {
    let file = File::open(input_txt)?;
    let reader = BufReader::new(file);

    let mut edges: Vec<(i64, i64)> = Vec::new();
    let mut nodes: BTreeSet<i64> = BTreeSet::new();

    for line in reader.lines() {
        let line = match line {
            Ok(s) => s,
            Err(_) => continue,
        };
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') { continue; }
        let mut it = s.split_whitespace();
        let u = match it.next().and_then(|x| x.parse::<i64>().ok()) { Some(x) => x, None => continue };
        let v = match it.next().and_then(|x| x.parse::<i64>().ok()) { Some(x) => x, None => continue };
        edges.push((u, v));
        nodes.insert(u);
        nodes.insert(v);
    }

    let keep: HashSet<i64> = nodes.into_iter().take(n_nodes).collect();

    let mut out = File::create(output_txt)?;
    for (u, v) in edges {
        if keep.contains(&u) && keep.contains(&v) {
            writeln!(out, "{} {}", u, v)?;
        }
    }
    Ok(())
}

/// -------- Chargement & remap --------
/// Charge "u v" (non orienté), remappe les IDs d'origine → 0..n-1.
/// Retourne (adj, map_back) où adj[u] = voisins de u, et map_back[new_id] = original_id.
fn load_graph_remap(path: &str) -> (Vec<Vec<usize>>, Vec<i64>) {
    let f = File::open(path).expect("Impossible d'ouvrir le fichier en entrée");
    let r = BufReader::new(f);

    let mut edges: Vec<(i64, i64)> = Vec::new();
    let mut ids: BTreeSet<i64> = BTreeSet::new();

    for line in r.lines() {
        let line = match line {
            Ok(s) => s,
            Err(_) => continue,
        };
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') { continue; }
        let mut it = s.split_whitespace();
        let u = match it.next().and_then(|x| x.parse::<i64>().ok()) { Some(x) => x, None => continue };
        let v = match it.next().and_then(|x| x.parse::<i64>().ok()) { Some(x) => x, None => continue };
        edges.push((u, v));
        ids.insert(u);
        ids.insert(v);
    }

    // IDs triés → map 0..n-1
    let ids_vec: Vec<i64> = ids.into_iter().collect();
    let n = ids_vec.len();

    let mut to_new: HashMap<i64, usize> = HashMap::with_capacity(n);
    for (i, &orig) in ids_vec.iter().enumerate() { to_new.insert(orig, i); }

    let mut adj = vec![Vec::<usize>::new(); n];
    for (u0, v0) in edges {
        let u = *to_new.get(&u0).expect("ID absent du mapping");
        let v = *to_new.get(&v0).expect("ID absent du mapping");
        adj[u].push(v);
        adj[v].push(u); // non orienté
    }
    (adj, ids_vec)
}

/// -------- Matrice NEXT-HOP (BFS répété, graphe non pondéré) --------
/// next[i,j] = premier sommet après i vers j ; -1 si pas de chemin. Convention: next[i,i] = i.
fn next_hop_matrix(adj: &[Vec<usize>]) -> Vec<i32> {
    let n = adj.len();
    let mut next = vec![NOHOP; n * n];

    for s in 0..n {
        next[idx(n, s, s)] = s as i32;

        let mut q = VecDeque::new();
        let mut visited = vec![false; n];
        let mut first_hop = vec![NOHOP; n];

        visited[s] = true;
        // distance 1 : next-hop = le voisin lui-même
        for &v in &adj[s] {
            if !visited[v] {
                visited[v] = true;
                next[idx(n, s, v)] = v as i32;
                first_hop[v] = v as i32;
                q.push_back(v);
            }
        }

        // BFS
        while let Some(u) = q.pop_front() {
            for &w in &adj[u] {
                if !visited[w] {
                    visited[w] = true;
                    // hérite du premier saut depuis s
                    next[idx(n, s, w)] = if u == s { w as i32 } else { first_hop[u] };
                    first_hop[w] = next[idx(n, s, w)];
                    q.push_back(w);
                }
            }
        }
    }
    next
}

/// -------- Sauvegardes CSV --------
fn save_next_hop_csv(next: &[i32], n: usize, path: &str) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    for i in 0..n {
        for j in 0..n {
            if j > 0 { write!(f, ",")?; }
            write!(f, "{}", next[idx(n, i, j)])?;
        }
        writeln!(f)?;
    }
    Ok(())
}

fn save_mapping_csv(map_back: &[i64], path: &str) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    writeln!(f, "new_id,original_id")?;
    for (new_id, &orig) in map_back.iter().enumerate() {
        writeln!(f, "{},{}", new_id, orig)?;
    }
    Ok(())
}

/// -------- main --------
/// Args: <input_txt> [n_nodes]
fn main() -> std::io::Result<()> {
    // Par défaut, on lit le fichier placé dans src/
    let default_input = "src/roadNet-CA.txt";
    let input = std::env::args().nth(1).unwrap_or_else(|| default_input.to_string());
    let n_nodes: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(1830);

    ensure_exists(&input);

    // 1) Prétraitement → data_pretraitee.txt
    preprocess_first_n_nodes(&input, "data_pretraitee.txt", n_nodes)?;
    eprintln!("Prétraitement OK → data_pretraitee.txt");

    // 2) Chargement + next-hop
    let (adj, map_back) = load_graph_remap("data_pretraitee.txt");
    let n = adj.len();
    eprintln!("Sommets (uniques) retenus : {}", n);

    let next = next_hop_matrix(&adj);

    // 3) Sorties
    save_next_hop_csv(&next, n, "next_hop.csv")?;
    save_mapping_csv(&map_back, "node_mapping.csv")?;
    eprintln!("Écrit : next_hop.csv, node_mapping.csv");
    Ok(())
}
