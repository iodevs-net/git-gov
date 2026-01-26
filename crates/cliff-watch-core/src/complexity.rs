//! Complexity Engine - Análisis de entropía lógica mediante compresión
//!
//! Utiliza Zstd para estimar la complejidad algorítmica (Entropía de Kolmogorov).

use zstd::stream::encode_all;
use syn::{parse_file, Item};
use std::path::Path;

// === CONSTANTES CALIBRADAS ===
/// Peso de cada item semántico (función, struct, enum, impl)
pub const SEMANTIC_ITEM_WEIGHT: f64 = 10.0;

/// Techo máximo del score semántico (evita archivos gigantes dominar)
pub const MAX_SEMANTIC_SCORE: f64 = 50.0;

/// Multiplicador del ratio de compresión para normalizar a escala 0-100
pub const NCD_MULTIPLIER: f64 = 50.0;

/// Umbral bajo el cual el código se considera spam/boilerplate
pub const SPAM_THRESHOLD: f64 = 10.0;

/// Estima el "Costo Entrópico" total del código
pub fn estimate_entropic_cost(code: &str, file_path: Option<&Path>) -> f64 {
    if code.is_empty() {
        return 0.0;
    }

    let compression_score = calculate_compression_ratio(code) * NCD_MULTIPLIER;
    
    let semantic_score = if is_rust_file(file_path) {
        analyze_rust_semantics(code)
    } else {
        analyze_generic_complexity(code)
    };

    (compression_score + semantic_score).min(100.0).max(1.0)
}

/// Calcula el ratio de compresión (proxy de entropía de Kolmogorov)
/// Retorna 0.0 (muy comprimible) a 1.0 (alta entropía/aleatorio)
pub fn calculate_compression_ratio(code: &str) -> f64 {
    let bytes = code.as_bytes();
    if bytes.is_empty() { return 0.0; }
    
    let compressed = match encode_all(bytes, 3) {
        Ok(c) => c,
        Err(_) => return 0.5, // Fallback conservador
    };
    
    // Ratio: código repetitivo comprime bien (bajo), código único comprime mal (alto)
    (compressed.len() as f64 / bytes.len() as f64).min(1.0)
}

/// Analiza la densidad semántica de código Rust mediante AST
fn analyze_rust_semantics(code: &str) -> f64 {
    match parse_file(code) {
        Ok(file) => {
            let items_count = file.items.iter()
                .filter(|item| matches!(
                    item, 
                    Item::Fn(_) | Item::Struct(_) | Item::Enum(_) | Item::Impl(_)
                ))
                .count();
            
            (items_count as f64 * SEMANTIC_ITEM_WEIGHT).min(MAX_SEMANTIC_SCORE)
        },
        Err(_) => 0.0, // Rust inválido o fragmento incompleto
    }
}

/// Heurística genérica para archivos no-Rust
fn analyze_generic_complexity(code: &str) -> f64 {
    let mut unique_lines = std::collections::HashSet::new();
    let non_empty_lines = code.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            unique_lines.insert(l.trim());
            l
        })
        .count();
    
    // Penalización por repetición: Usar min(total, unique * 2)
    // Boilerplate repetitivo tendrá unique bajo.
    let effective_lines = (non_empty_lines as f64).min((unique_lines.len() as f64) * 3.0);

    // 1 línea significativa = 2 puntos de complejidad
    (effective_lines * 2.0).min(MAX_SEMANTIC_SCORE)
}

/// Detecta si un archivo es código Rust
fn is_rust_file(path: Option<&Path>) -> bool {
    path.and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .map(|e| e == "rs")
        .unwrap_or(false)
}

/// Detecta spam/boilerplate extremo
pub fn is_likely_spam(code: &str) -> bool {
    estimate_entropic_cost(code, None) < SPAM_THRESHOLD
}

/// [AVANZADO] NCD real para detectar copy-paste del repo
pub fn calculate_ncd_against_context(new_code: &str, existing_context: &str) -> f64 {
    let c_new = compress_size(new_code);
    let c_context = compress_size(existing_context);
    let c_combined = compress_size(&format!("{}{}", existing_context, new_code));
    
    let numerator = c_combined.saturating_sub(c_context.min(c_new)) as f64;
    let denominator = c_context.max(c_new) as f64;
    
    if denominator == 0.0 {
        return 1.0; // Contexto vacío, asumimos novedad máxima
    }
    
    (numerator / denominator).max(0.0).min(1.0)
}

fn compress_size(data: &str) -> usize {
    encode_all(data.as_bytes(), 3)
        .map(|v| v.len())
        .unwrap_or(data.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boilerplate_has_low_cost() {
        // Zstd has overhead for very short strings. Use a longer buffer.
        let code = "// TODO\n".repeat(100); 
        assert!(estimate_entropic_cost(&code, None) < SPAM_THRESHOLD + 15.0);
    }

    #[test]
    fn complex_rust_has_high_cost() {
        let code = r#"
            fn fibonacci(n: u32) -> u32 {
                match n {
                    0 => 0,
                    1 => 1,
                    _ => fibonacci(n-1) + fibonacci(n-2)
                }
            }
        "#;
        assert!(estimate_entropic_cost(code, Some(Path::new("test.rs"))) > 20.0);
    }

    #[test]
    fn ncd_detects_duplication() {
        let context = "fn hello() { println!(\"hello\"); }";
        let duplicate = "fn hello() { println!(\"hello\"); }";
        let novel = "fn goodbye() { println!(\"bye\"); }";
        
        let ncd_dup = calculate_ncd_against_context(duplicate, context);
        let ncd_new = calculate_ncd_against_context(novel, context);
        
        assert!(ncd_dup < ncd_new, "Duplicate should have lower NCD");
    }
}
