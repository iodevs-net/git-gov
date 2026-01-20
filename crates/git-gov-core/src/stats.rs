//! Estadísticas y métricas de validación para GitGov
//!
//! Este módulo implementa funciones estadísticas para validar contribuciones
//! humanas vs AI, incluyendo cálculo de burstiness, NCD y scores compuestos.

use statrs::statistics::Statistics;

/// Calcula la burstiness de una serie de tiempos de edición
/// Burstiness mide la variabilidad en los intervalos entre eventos
pub fn calculate_burstiness(times: &[f64]) -> f64 {
    if times.len() < 2 {
        return 0.0;
    }
    
    // Calcular intervalos entre eventos (usar valor absoluto para manejar datos no ordenados)
    let intervals: Vec<f64> = times.windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .collect();
    
    // Filtrar intervalos que son demasiado pequeños para evitar ruido numérico
    let filtered_intervals: Vec<f64> = intervals.iter()
        .filter(|&&x| x > 1e-10)  // Filtrar valores casi cero
        .cloned()
        .collect();
    
    // Si no hay intervalos válidos después del filtrado, retornar 0
    if filtered_intervals.is_empty() {
        return 0.0;
    }
    
    let mean = filtered_intervals.clone().mean();
    let std_dev = filtered_intervals.std_dev();
    
    // Manejar casos donde mean es muy pequeño o std_dev es cero
    if mean < 1e-10 {
        return -1.0; // Máxima regularidad si no hay tiempo transcurrido
    }

    if std_dev < 1e-10 {
        return -1.0; // Regularidad perfecta
    }
    
    // Burstiness = (std_dev - mean) / (std_dev + mean)
    // Según Kleinberg et al., este valor está en el rango [-1, 1]
    // 1: Altamente ráfaga (humano), -1: Completamente regular (máquina)
    let burstiness = (std_dev - mean) / (std_dev + mean);
    
    // Validar que el resultado sea finito
    if burstiness.is_finite() {
        burstiness.clamp(-1.0, 1.0)
    } else {
        -1.0
    }
}

/// Calcula la distancia de compresión normalizada (NCD) entre dos secuencias
/// NCD mide la similitud entre patrones de código
pub fn calculate_ncd(x: &[u8], y: &[u8]) -> f64 {
    use zstd::stream::encode_all;
    
    // Comprimir individualmente
    let cx = encode_all(x, 1).unwrap_or_else(|_| x.to_vec());
    let cy = encode_all(y, 1).unwrap_or_else(|_| y.to_vec());
    
    // Comprimir concatenación
    let mut concat = x.to_vec();
    concat.extend_from_slice(y);
    let cxy = encode_all(&*concat, 1).unwrap_or_else(|_| concat);
    
    // NCD = (cxy - min(cx, cy)) / max(cx, cy)
    let min_c = cx.len().min(cy.len()) as f64;
    let max_c = cx.len().max(cy.len()) as f64;
    let cxy_len = cxy.len() as f64;
    
    if max_c == 0.0 {
        return 0.0;
    }
    
    ((cxy_len - min_c) / max_c).max(0.0).min(1.0)
}

/// Calcula un score compuesto que representa la probabilidad de contribución humana
/// Score = 0.0 (AI) a 1.0 (humano)
pub fn calculate_human_score(burstiness: f64, ncd: f64) -> f64 {
    // Normalizar burstiness: mapear de [-1, 1] a [0, 1]
    // B = 1 (Burst/Human) -> 1.0
    // B = -1 (Regular/Machine) -> 0.0
    let norm_burstiness = (burstiness + 1.0) / 2.0;
    
    // NCD ya está en rango 0-1
    let norm_ncd = ncd.clamp(0.0, 1.0);
    
    // Score compuesto: 70% burstiness + 30% NCD
    // (Burstiness es más indicativo de comportamiento humano)
    0.7 * norm_burstiness + 0.3 * norm_ncd
}

/// Valida si un score representa una contribución humana
pub fn validate_human_contribution(score: f64, threshold: f64) -> bool {
    score >= threshold
}

/// Valida si un score representa una contribución AI
pub fn validate_ai_contribution(score: f64, threshold: f64) -> bool {
    score <= threshold
}

/// Calcula un threshold dinámico basado en scores históricos
pub fn calculate_dynamic_threshold(historical_scores: &[f64], base_threshold: f64) -> f64 {
    if historical_scores.is_empty() {
        return base_threshold;
    }
    
    let avg_score = historical_scores.iter().sum::<f64>() / historical_scores.len() as f64;
    
    // Threshold dinámico: 90% del promedio histórico
    // Esto adapta el threshold a los patrones observados
    avg_score * 0.9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burstiness_calculation() {
        // Datos con ráfagas (distribución de intervalos muy desigual)
        // Intervalos: [0.1, 10.0, 0.1, 10.0] -> mu=5.05, sigma=5.71
        // B = (5.71 - 5.05) / (5.71 + 5.05) = 0.06 (positivo = bursty)
        let human_times = vec![0.0, 0.1, 10.1, 10.2, 20.2];
        let burstiness = calculate_burstiness(&human_times);
        assert!(burstiness > 0.0, "Positive burstiness expected for irregular patterns");
        
        // Datos con baja variabilidad (AI-like / Regular)
        // Intervalos: [1.0, 1.0, 1.0, 1.0] -> mu=1.0, sigma=0.0
        // B = -1.0
        let ai_times = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let burstiness = calculate_burstiness(&ai_times);
        assert!(burstiness < -0.9, "Negative burstiness expected for regular AI patterns");
    }

    #[test]
    fn test_ncd_calculation() {
        // Datos idénticos
        let x = b"identical identical identical";
        let y = b"identical identical identical";
        let ncd = calculate_ncd(x, y);
        assert!(ncd < 0.5, "Low NCD expected for identical data");
        
        // Datos diferentes
        let x = b"completely different data";
        let y = b"unrelated content here";
        let ncd = calculate_ncd(x, y);
        assert!(ncd > 0.5, "High NCD expected for different data");
    }

    #[test]
    fn test_human_score_calculation() {
        // Alto burstiness (0.9) + bajo NCD (0.2) = alto score humano
        // norm_B = (0.9+1)/2 = 0.95. score = 0.7*0.95 + 0.3*0.2 = 0.665 + 0.06 = 0.725
        let score = calculate_human_score(0.9, 0.2);
        assert!(score > 0.6, "High human score expected for bursty patterns");
        
        // Bajo burstiness (-0.9) + medio NCD (0.5) = bajo score humano
        // norm_B = (-0.9+1)/2 = 0.05. score = 0.7*0.05 + 0.3*0.5 = 0.035 + 0.15 = 0.185
        let score = calculate_human_score(-0.9, 0.5);
        assert!(score < 0.4, "Low human score expected for regular patterns");
    }

    #[test]
    fn test_dynamic_threshold() {
        let historical = vec![0.85, 0.88, 0.78, 0.92];
        let threshold = calculate_dynamic_threshold(&historical, 0.7);
        assert!(threshold > 0.7, "Dynamic threshold should adapt upwards");
    }

    #[test]
    fn test_debug_values() {
        // Debug burstiness values
        let human_times = vec![10.0, 0.1, 0.1, 20.0, 0.1, 0.1, 30.0];
        let human_burstiness = calculate_burstiness(&human_times);
        println!("Human burstiness: {}", human_burstiness);
        
        let ai_times = vec![1.0, 1.01, 0.99, 1.0, 1.0, 1.01];
        let ai_burstiness = calculate_burstiness(&ai_times);
        println!("AI burstiness: {}", ai_burstiness);
        
        // Debug human score values
        let high_score = calculate_human_score(0.9, 0.2);
        println!("High human score (0.9, 0.2): {}", high_score);
        
        let low_score = calculate_human_score(0.1, 0.8);
        println!("Low human score (0.1, 0.8): {}", low_score);
        
        // These should always pass
        assert!(true);
    }
}