//! Complexity Engine - Análisis de entropía lógica mediante compresión
//!
//! Utiliza Zstd para estimar la complejidad algorítmica (Entropía de Kolmogorov).
//! Un código difícil de comprimir indica alta densidad de información.

use zstd::stream::encode_all;
use syn::{parse_file, Item};

/// Estima el "Costo Entrópico" del código basándose en el ratio de compresión
/// y la densidad semántica (AST).
pub fn estimate_entropic_cost(code: &str) -> f64 {
    let bytes = code.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }

    // 1. Entropía de Kolmogorov (Compresión)
    let compressed = match encode_all(bytes, 3) {
        Ok(c) => c,
        Err(_) => return 0.5, 
    };
    let compression_ratio = compressed.len() as f64 / bytes.len() as f64;

    // 2. Syntactic Burstiness (Análisis AST si es Rust)
    let semantic_weight = match parse_file(code) {
        Ok(file) => {
            // Contamos items significativos (Funciones, Structs, Enums)
            let items_count = file.items.iter().filter(|item| {
                matches!(item, Item::Fn(_) | Item::Struct(_) | Item::Enum(_) | Item::Impl(_))
            }).count();
            
            // Si hay estructura semántica, el peso sube. 
            // Un archivo con 5 funciones es más denso que uno con solo comentarios.
            (items_count as f64 * 10.0).min(50.0)
        },
        Err(_) => 0.0, // No es código Rust válido o Fragmento incompleto
    };

    let total_score = (compression_ratio * 50.0) + semantic_weight;
    
    (total_score).min(100.0).max(1.0)
}

/// Identifica si el código parece "Spam de IA" basándose en la regularidad estructural.
pub fn is_likely_spam(code: &str) -> bool {
    let cost = estimate_entropic_cost(code);
    // Un costo extremadamente bajo (<10) suele indicar boilerplate, spam repetitivo
    // o código que no tiene estructura lógica válida.
    cost < 10.0
}

