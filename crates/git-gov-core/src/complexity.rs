//! Complexity Engine - Análisis de entropía lógica mediante compresión
//!
//! Utiliza Zstd para estimar la complejidad algorítmica (Entropía de Kolmogorov).
//! Un código difícil de comprimir indica alta densidad de información.

use zstd::stream::encode_all;

/// Estima el "Costo Entrópico" del código basándose en el ratio de compresión.
/// 
/// El costo representa cuántos "Créditos de Atención" debe pagar el desarrollador
/// para validar este fragmento.
pub fn estimate_entropic_cost(code: &str) -> f64 {
    let bytes = code.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }

    // Usamos un nivel de compresión alto para encontrar patrones profundos
    let compressed = match encode_all(bytes, 3) {
        Ok(c) => c,
        Err(_) => return 0.5, 
    };

    let ratio = compressed.len() as f64 / bytes.len() as f64;
    
    // El costo escala con la densidad de información. 
    // Un archivo muy denso (ratio > 0.4) tiene un costo exponencialmente mayor.
    let base_cost = (ratio * 50.0).max(1.0);
    
    // Si el código es largo y denso, el costo sube.
    (base_cost).min(100.0) 
}

/// Identifica si el código parece "Spam de IA" basándose en la regularidad estructural.
pub fn is_likely_spam(code: &str) -> bool {
    let cost = estimate_entropic_cost(code);
    // Un costo extremadamente bajo suele indicar boilerplate o spam repetitivo.
    cost < 5.0
}

