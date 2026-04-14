use anomaly_detector::rcf::RcfTree;

fn main() {
    let mut tree = RcfTree::new();
    
    println!("=== Вставляем нормальные значения (10) ===");
    for i in 0..20 {
        let depth = tree.insert(10.0);
        println!("{}: depth={}", i, depth);
    }
    
    println!("\n=== Вставляем аномалию (100) ===");
    let depth = tree.insert(100.0);
    println!("anomaly depth={}", depth);
    
    println!("\n=== Вставляем еще нормальные ===");
    for i in 0..10 {
        let depth = tree.insert(10.0);
        println!("{}: depth={}", i, depth);
    }
}