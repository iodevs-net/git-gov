use proptest::prelude::*;
use git_gov_core::monitor::AttentionBattery;
use git_gov_core::complexity::estimate_entropic_cost;
use std::time::Duration;

proptest! {
    /// Invariante de Rango de la Batería
    /// No importa cuánta carga o tiempo pase, el nivel debe ser 0 <= level <= capacity
    #[test]
    fn battery_level_invariants(
        initial_charge in 0.0..100.0,
        motor_entropy in 0.0..2.0, // Incluimos valores fuera de rango para stress
        seconds in 0u64..3600u64,  // Hasta 1 hora
        events in 0usize..10000usize
    ) {
        let mut battery = AttentionBattery::new();
        battery.level = initial_charge;
        
        battery.charge(motor_entropy, Duration::from_secs(seconds), events);
        
        prop_assert!(battery.level >= 0.0, "Battery level underflow: {}", battery.level);
        prop_assert!(battery.level <= battery.capacity + 1e-10, "Battery level overflow: {}", battery.level);
    }

    /// Invariante de Causalidad
    /// Si no hay eventos de hardware, la batería solo puede decaer, nunca subir.
    #[test]
    fn battery_causality_invariant(
        initial_charge in 10.0..100.0,
        motor_entropy in 0.0..1.0,
        seconds in 1u64..60u64
    ) {
        let mut battery = AttentionBattery::new();
        battery.level = initial_charge;
        battery.causal_event_count = 100; // Seteamos un base
        
        // Intentamos cargar con 0 eventos adicionales (spoofing)
        battery.charge(motor_entropy, Duration::from_secs(seconds), 100);
        
        prop_assert!(battery.level <= initial_charge + 1e-10, "Battery increased without hardware events!");
    }

    /// Invariante de Acotamiento del Costo Entrópico
    /// Cualquier string, por muy caótico o largo que sea, debe tener un costo finito y acotado.
    #[test]
    fn entropic_cost_is_bounded(s in ".{0,10000}") {
        let cost = estimate_entropic_cost(&s);
        prop_assert!(cost >= 0.0);
        prop_assert!(cost <= 100.0, "Cost exceeded limit: {}", cost);
    }

    /// Estabilidad del Costo
    /// Pequeños cambios en el código no deben causar saltos masivos en el costo.
    #[test]
    fn entropic_cost_stability(s in ".{100,500}") {
        let cost1 = estimate_entropic_cost(&s);
        let s2 = format!("{} ", s); // Añadimos un espacio
        let cost2 = estimate_entropic_cost(&s2);
        
        let diff = (cost1 - cost2).abs();
        prop_assert!(diff < 5.0, "Cost is unstable! Diff: {} for similar strings", diff);
    }
}
