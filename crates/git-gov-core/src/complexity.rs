//! Complexity Engine - Análisis de entropía lógica mediante compresión
//!
//! Utiliza Zstd para estimar la complejidad algorítmica (Entropía de Kolmogorov).
//! Un código difícil de comprimir indica alta densidad de información.

use zstd::stream::encode_all;

/// Estima la entropía lógica basándose en el ratio de compresión.
/// 
/// Retorna un valor entre 0.0 (totalmente repetitivo) y 1.0 (máxima complejidad).
pub fn estimate_code_complexity(code: &str) -> f64 {
    let bytes = code.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }

    // Usamos un nivel de compresión alto para encontrar patrones profundos
    let compressed = match encode_all(bytes, 3) {
        Ok(c) => c,
        Err(_) => return 0.5, // Fallback neutral
    };

    let ratio = compressed.len() as f64 / bytes.len() as f64;
    
    // Normalizamos: en código fuente, un ratio de 0.4-0.5 es complejidad media-alta.
    // Mapeamos ratio 0.4 a ~0.6 de complejidad.
    (ratio * 1.2).min(1.0)
}

/// Identifica si el código parece "Spam de IA" basándose en la regularidad estructural.
pub fn is_likely_spam(code: &str) -> bool {
    let complexity = estimate_code_complexity(code);
    // Un código extremadamente repetitivo (< 0.15) suele ser boilerplate o spam.
    complexity < 0.15
}

