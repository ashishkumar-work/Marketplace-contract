use soroban_sdk::{contractimpl, Bytes, Env, Symbol, Address};

#[derive(Copy, Clone)]
enum AuthorizationLevel {
    Basic,           // Simple verification 
    Intermediate,    // More in-depth checks
    Advanced,        // Highest level of verification 
}

struct Producer {
    address: Address,
    authorization_level: AuthorizationLevel,

}

#[contractimpl]
pub trait EnergyProducerVerification {
    // State Management
    fn producers(env: Env) -> Vec<Producer> {
        env.data().get(Symbol::from_str("producers")).unwrap_or(Ok(Vec::new())).unwrap()
    }

    fn register_producer(env: Env, producer: Producer) {
        

        let mut producers = Self::producers(env.clone());
        producers.push(producer);
        env.data().set(Symbol::from_str("producers"), &producers);
    }

    fn update_authorization_level(env: Env, address: Address, level: AuthorizationLevel) {
        

        let mut producers = Self::producers(env.clone());
        if let Some(p) = producers.iter_mut().find(|p| p.address == address) {
            p.authorization_level = level;
            env.data().set(Symbol::from_str("producers"), &producers); 
        } else {
            panic!("Producer not found")
        }
    }


    fn is_authorized_producer(env: Env, address: Address) -> bool {
        Self::producers(env).iter().any(|p| p.address == address)
    }
}
