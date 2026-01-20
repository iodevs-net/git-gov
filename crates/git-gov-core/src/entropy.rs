use std::io::Cursor;
use zstd::stream::encode_all;

/// Calcula el Normalized Compression Distance (NCD) entre dos secuencias
/// Usa compresión zstd para estimar la distancia
pub fn calculate_ncd(x: &[u8], y: &[u8]) -> f64 {
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

/// Calcula la entropía aproximada de una secuencia de bytes
pub fn calculate_entropy(data: &[u8]) -> f64 {
    let mut frequency = [0u64; 256];
    
    // Contar frecuencias de bytes
    for &byte in data {
        frequency[byte as usize] += 1;
    }
    
    let data_len = data.len() as f64;
    let mut entropy = 0.0;
    
    // Calcular entropía usando la fórmula de Shannon
    for &count in &frequency {
        if count > 0 {
            let probability = count as f64 / data_len;
            entropy -= probability * probability.log2();
        }
    }
    
    entropy
}

/// Calcula la entropía normalizada (0-1)
pub fn calculate_normalized_entropy(data: &[u8]) -> f64 {
    let entropy = calculate_entropy(data);
    // Normalizar por la entropía máxima posible (8 bits por byte)
    entropy / 8.0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ncd_calculation() {
        let x = b"hello world hello world";
        let y = b"hello world goodbye world";
        let ncd = calculate_ncd(x, y);
        assert!(ncd >= 0.0 && ncd <= 1.0, "NCD should be in range [0, 1]");
    }
    
    #[test]
    fn test_entropy_calculation() {
        let data = b"test data";
        let entropy = calculate_entropy(data);
        assert!(entropy >= 0.0 && entropy <= 8.0, "Entropy should be in range [0, 8]");
    }
    
    #[test]
    fn test_normalized_entropy() {
        let data = b"test data";
        let entropy = calculate_normalized_entropy(data);
        assert!(entropy >= 0.0 && entropy <= 1.0, "Normalized entropy should be in range [0, 1]");
    }
}