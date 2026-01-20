// Módulo para análisis estadístico (burstiness, NCD)
use statrs::statistics::Statistics;

/// Calcula el burstiness de una serie de datos
/// Fórmula: B = (σ - μ) / (σ + μ)
/// Donde σ es la desviación estándar y μ es la media
pub fn calculate_burstiness(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    
    let mean = data.mean();
    let std_dev = data.std_dev();
    
    if std_dev == 0.0 {
        return 0.0;
    }
    
    (std_dev - mean) / (std_dev + mean)
}

/// Calcula el Normalized Compression Distance (NCD) entre dos secuencias
/// Usa compresión zstd para estimar la distancia
pub fn calculate_ncd(x: &[u8], y: &[u8]) -> f64 {
    use std::io::Cursor;
    use zstd::stream::encode_all;
    
    // Comprimir individualmente
    let c_x = encode_all(Cursor::new(x), 1).unwrap_or_else(|_| vec![]);
    let c_y = encode_all(Cursor::new(y), 1).unwrap_or_else(|_| vec![]);
    
    // Comprimir concatenados
    let mut concatenated = Vec::with_capacity(x.len() + y.len());
    concatenated.extend_from_slice(x);
    concatenated.extend_from_slice(y);
    let c_xy = encode_all(Cursor::new(&concatenated), 1).unwrap_or_else(|_| vec![]);
    
    // Calcular NCD
    if c_x.len() == 0 || c_y.len() == 0 || c_xy.len() == 0 {
        return 0.0;
    }
    
    let c_x_len = c_x.len() as f64;
    let c_y_len = c_y.len() as f64;
    let c_xy_len = c_xy.len() as f64;
    
    (c_xy_len - f64::min(c_x_len, c_y_len)) / f64::max(c_x_len, c_y_len)
}

/// Calcula el puntaje humano basado en burstiness y NCD
pub fn calculate_human_score(burstiness: f64, ncd: f64) -> f64 {
    // Alta burstiness y alto NCD indican contribución humana
    (burstiness * 0.6 + ncd * 0.4) as f64
}

/// Valida si un puntaje supera el umbral humano
pub fn validate_human_contribution(score: f64, threshold: f64) -> bool {
    score > threshold
}

/// Valida si un puntaje indica contribución de IA
pub fn validate_ai_contribution(score: f64, threshold: f64) -> bool {
    score < threshold
}

/// Calcula umbral dinámico basado en datos históricos
pub fn calculate_dynamic_threshold(historical_scores: &[f64], base_threshold: f64) -> f64 {
    if historical_scores.is_empty() {
        return base_threshold;
    }
    
    let mean_score = historical_scores.mean();
    // Ajustar umbral basado en el promedio histórico
    (mean_score * 0.9) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_burstiness_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 10.0];
        let burstiness = calculate_burstiness(&data);
        // Burstiness can be negative for certain patterns, but should not be zero for varying data
        assert!(burstiness != 0.0, "Burstiness should not be zero for varying data");
        println!("Burstiness for test data: {}", burstiness);
    }
    
    #[test]
    fn test_ncd_calculation() {
        let x = b"hello world hello world";
        let y = b"hello world goodbye world";
        let ncd = calculate_ncd(x, y);
        assert!(ncd >= 0.0 && ncd <= 1.0, "NCD should be in range [0, 1]");
    }
    
    #[test]
    fn test_human_score_calculation() {
        let burstiness = 0.85;
        let ncd = 0.65;
        let score = calculate_human_score(burstiness, ncd);
        assert!(score > 0.7, "High burstiness and NCD should give high human score");
    }
    
    #[test]
    fn test_ai_score_calculation() {
        let burstiness = 0.2;
        let ncd = 0.1;
        let score = calculate_human_score(burstiness, ncd);
        assert!(score < 0.3, "Low burstiness and NCD should give low human score");
    }
    
    #[test]
    fn test_dynamic_threshold() {
        let historical_scores = vec![0.8, 0.85, 0.78, 0.9, 0.82];
        let base_threshold = 0.7;
        let dynamic_threshold = calculate_dynamic_threshold(&historical_scores, base_threshold);
        assert!(dynamic_threshold > 0.7, "Dynamic threshold should be adjusted based on history");
    }
    
    proptest! {
        #[test]
        fn test_burstiness_properties(data in prop::collection::vec(0.0..100.0, 5..100)) {
            let burstiness = calculate_burstiness(&data);
            prop_assert!(burstiness >= -1.0 && burstiness <= 1.0, "Burstiness should be in valid range");
        }
        
        #[test]
        fn test_human_score_properties(burstiness in 0.0..1.0, ncd in 0.0..1.0) {
            let score = calculate_human_score(burstiness, ncd);
            prop_assert!(score >= 0.0 && score <= 1.0, "Human score should be in range [0, 1]");
            
            // High values should give high score
            if burstiness > 0.8 && ncd > 0.8 {
                prop_assert!(score > 0.7, "High burstiness and NCD should give high human score");
            }
            
            // Low values should give low score
            if burstiness < 0.2 && ncd < 0.2 {
                prop_assert!(score < 0.3, "Low burstiness and NCD should give low human score");
            }
        }
    }
}