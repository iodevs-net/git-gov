use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub x: f64,
    pub y: f64,
    pub timestamp: u128,
}

#[derive(Debug, Clone)]
pub struct MouseSentinel {
    pub event_buffer: Arc<Mutex<VecDeque<MouseEvent>>>,
    pub max_buffer_size: usize,
}

impl MouseSentinel {
    pub fn new(max_buffer_size: usize) -> Self {
        Self {
            event_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(max_buffer_size))),
            max_buffer_size,
        }
    }

    pub fn capture_event(&self, x: f64, y: f64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let mut buffer = self.event_buffer.lock().unwrap();
        buffer.push_back(MouseEvent { x, y, timestamp });
        
        if buffer.len() > self.max_buffer_size {
            buffer.pop_front();
        }
    }

    pub fn analyze_events(&self) -> Result<KinematicMetrics, MouseSentinelError> {
        let buffer = self.event_buffer.lock().unwrap();
        if buffer.len() < 3 {
            return Err(MouseSentinelError::InsufficientData);
        }

        let positions: Vec<(f64, f64)> = buffer.iter().map(|event| (event.x, event.y)).collect();
        let timestamps: Vec<u128> = buffer.iter().map(|event| event.timestamp).collect();

        let ldlj = self.calculate_ldlj(&positions, &timestamps)?;
        let spec_entropy = self.calculate_spectral_entropy(&positions)?;
        let path_entropy = self.calculate_curvature_entropy(&positions)?;
        let throughput = self.calculate_throughput(&positions, &timestamps)?;

        Ok(KinematicMetrics {
            ldlj,
            spec_entropy,
            path_entropy,
            throughput,
        })
    }

    fn calculate_ldlj(&self, positions: &[(f64, f64)], timestamps: &[u128]) -> Result<f64, MouseSentinelError> {
        if positions.len() < 3 {
            return Err(MouseSentinelError::InsufficientData);
        }

        let mut jerk_samples = Vec::new();
        let mut velocities = Vec::new();
        let mut accelerations = Vec::new();

        for i in 1..positions.len() {
            let dx = positions[i].0 - positions[i-1].0;
            let dy = positions[i].1 - positions[i-1].1;
            let dt = (timestamps[i] - timestamps[i-1]) as f64 / 1000.0; // Convert to seconds
             
            if dt > 0.0 {
                let velocity = (dx.powi(2) + dy.powi(2)).sqrt() / dt;
                velocities.push(velocity);
            }
        }

        for i in 1..velocities.len() {
            let dv = velocities[i] - velocities[i-1];
            let dt = (timestamps[i+1] - timestamps[i]) as f64 / 1000.0;
             
            if dt > 0.0 {
                let acceleration = dv / dt;
                accelerations.push(acceleration);
            }
        }

        for i in 1..accelerations.len() {
            let da = accelerations[i] - accelerations[i-1];
            let dt = (timestamps[i+2] - timestamps[i+1]) as f64 / 1000.0;
             
            if dt > 0.0 {
                let jerk = da / dt;
                jerk_samples.push(jerk);
            }
        }

        if jerk_samples.is_empty() {
            return Err(MouseSentinelError::InsufficientData);
        }

        let peak_velocity = velocities.iter().fold(0.0f64, |a, &b| a.max(b));
        let duration = (timestamps[timestamps.len()-1] - timestamps[0]) as f64 / 1000.0;
         
        println!("DEBUG LDLJ: peak_velocity={}, duration={}, jerk_samples_len={}", peak_velocity, duration, jerk_samples.len());
        
        let integral: f64 = jerk_samples.iter().map(|&j| j.powi(2)).sum();
        
        if peak_velocity == 0.0 {
            return Err(MouseSentinelError::Analysis("Peak velocity is zero".to_string()));
        }
        
        // Handle case where integral is zero (no jerk detected)
        let ldlj = if integral == 0.0 {
            // Return a default value for smooth movement (no jerk)
            0.0
        } else {
            let calculated_ldlj = -((integral * duration.powi(3)) / peak_velocity.powi(2)).ln();
            
            if !calculated_ldlj.is_finite() {
                return Err(MouseSentinelError::Analysis("LDLJ calculation resulted in non-finite value".to_string()));
            }
            calculated_ldlj
        };
        
        Ok(ldlj)
    }

    fn calculate_spectral_entropy(&self, positions: &[(f64, f64)]) -> Result<f64, MouseSentinelError> {
        if positions.len() < 3 {
            return Err(MouseSentinelError::InsufficientData);
        }

        let velocities: Vec<f64> = positions.windows(2)
            .map(|w| {
                let dx = w[1].0 - w[0].0;
                let dy = w[1].1 - w[0].1;
                (dx.powi(2) + dy.powi(2)).sqrt()
            })
            .collect();

        let total_velocity: f64 = velocities.iter().sum();
        if total_velocity == 0.0 {
            return Err(MouseSentinelError::InsufficientData);
        }

        let probabilities: Vec<f64> = velocities.iter()
            .map(|&v| v / total_velocity)
            .filter(|&p| p > 0.0)
            .collect();

        let entropy: f64 = probabilities.iter()
            .map(|&p| -p * p.log2())
            .sum();

        Ok(entropy)
    }

    fn calculate_curvature_entropy(&self, positions: &[(f64, f64)]) -> Result<f64, MouseSentinelError> {
        if positions.len() < 3 {
            return Err(MouseSentinelError::InsufficientData);
        }

        let curvatures: Vec<f64> = positions.windows(3)
            .map(|triplet| {
                let (a, b, c) = (triplet[0], triplet[1], triplet[2]);
                let area = 0.5 * ((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)).abs();
                let ab = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
                let bc = ((c.0 - b.0).powi(2) + (c.1 - b.1).powi(2)).sqrt();
                let ca = ((a.0 - c.0).powi(2) + (a.1 - c.1).powi(2)).sqrt();
                 
                if area > 0.0 && ab > 0.0 && bc > 0.0 && ca > 0.0 {
                    (4.0 * area) / (ab * bc * ca)
                } else {
                    0.0
                }
            })
            .collect();

        let total_curvature: f64 = curvatures.iter().sum();
        
        // Handle case where total curvature is zero (straight line movement)
        let entropy = if total_curvature == 0.0 {
            // Return a default value for straight line (minimum entropy)
            0.0
        } else {
            let probabilities: Vec<f64> = curvatures.iter()
                .map(|&c| c / total_curvature)
                .filter(|&p| p > 0.0)
                .collect();
                
            probabilities.iter()
                .map(|&p| -p * p.log2())
                .sum()
        };

        Ok(entropy)
    }

    fn calculate_throughput(&self, positions: &[(f64, f64)], timestamps: &[u128]) -> Result<f64, MouseSentinelError> {
        if positions.len() < 2 {
            return Err(MouseSentinelError::InsufficientData);
        }

        let start_pos = positions[0];
        let end_pos = positions[positions.len()-1];
        let distance = ((end_pos.0 - start_pos.0).powi(2) + (end_pos.1 - start_pos.1).powi(2)).sqrt();
        let movement_time = (timestamps[timestamps.len()-1] - timestamps[0]) as f64 / 1000.0;
         
        if movement_time == 0.0 {
            return Err(MouseSentinelError::InsufficientData);
        }

        let effective_id = (distance / 100.0).log2() + 1.0; // Simplified ID_e calculation
        let throughput = effective_id / movement_time;

        Ok(throughput)
    }
}

#[derive(Debug, Clone)]
pub struct KinematicMetrics {
    pub ldlj: f64,
    pub spec_entropy: f64,
    pub path_entropy: f64,
    pub throughput: f64,
}

#[derive(Debug, Error)]
pub enum MouseSentinelError {
    #[error("Insufficient data for analysis")]
    InsufficientData,
    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl From<String> for MouseSentinelError {
    fn from(s: String) -> Self {
        MouseSentinelError::Analysis(s)
    }
}