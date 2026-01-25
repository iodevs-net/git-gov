//! Test de Acoplamiento Cognitivo-Motor
//!
//! Valida la teoría de que la complejidad del código debe estar sincronizada
//! con el esfuerzo biomecánico del desarrollador.

use cliff_craft_core::complexity::estimate_entropic_cost;
use cliff_craft_core::stats::calculate_coupling_score;

#[test]
fn test_cognitive_coupling_logic() {
    // Escenario 1: Humano trabajando en código complejo
    // Alta complejidad lógica (~0.6) + Alta entropía motora detectiva (~0.5)
    let complex_code = "fn solve_cryptography(data: &[u8]) -> Result<Hash, Error> { 
        let mut hasher = Sha256::new();
        for chunk in data.chunks(64) {
            hasher.update(process_nonlinear(chunk));
        }
        Ok(hasher.finalize())
    }";
    
    // estimate_entropic_cost devuelve un valor entre 0.0 y 100.0
    // Normalizamos a 0.0-1.0 para el coupling
    let entropic_cost = estimate_entropic_cost(complex_code);
    let code_comp = entropic_cost / 100.0;
    let motor_ent = 0.55; // Humano: alta variabilidad
    let coupling = calculate_coupling_score(code_comp, motor_ent);
    
    println!("Humano - Comp: {:.2}, Motor: {:.2}, Coupling: {:.2}", code_comp, motor_ent, coupling);
    assert!(coupling > 0.4, "Trabajo humano debería tener acoplamiento razonable, got: {}", coupling);

    // Escenario 2: IA/Bot inyectando código complejo
    // El bot tiene entropía motora bajísima porque es robótico
    let motor_ent_bot = 0.05; 
    let coupling_bot = calculate_coupling_score(code_comp, motor_ent_bot);
    
    println!("Bot - Comp: {:.2}, Motor: {:.2}, Coupling: {:.2}", code_comp, motor_ent_bot, coupling_bot);
    // El coupling del bot debe ser significativamente menor que el del humano
    // (detecta que hay menos sincronía entre complejidad y esfuerzo motor)
    assert!(coupling_bot < coupling, 
        "Inyección de IA debería mostrar menor acoplamiento que humano: bot={}, human={}", 
        coupling_bot, coupling);
}

#[test]
fn test_spam_detection() {
    let spam_code = "// test test test test test\n// test test test test test\n".repeat(10);
    let is_spam = cliff_craft_core::complexity::is_likely_spam(&spam_code);
    assert!(is_spam, "Código altamente repetitivo debería marcarse como spam");
}
