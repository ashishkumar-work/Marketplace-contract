use soroban_sdk::{contractimpl, Bytes, Env, Symbol, BigInt, Address, IntoVal};

#[derive(Clone)]
struct Token {
    id: u64,
    issuer: Address,
    name: Bytes,
    description: Bytes, 
    supply: BigInt,     
}

#[contractimpl]
pub trait SEP10Token {
    // State Management
    fn tokens(env: Env) -> Vec<Token> {
        env.data().get(Symbol::from_str("tokens")).unwrap_or(Ok(Vec::new())).unwrap()
    }

    fn new(env: Env, token: Token) {
        let caller = env.invoker();
        if caller != token.issuer {
            panic!("Only the issuer can create new tokens"); 
        }

        let mut tokens = Self::tokens(env.clone());
        tokens.push(token);
        env.data().set(Symbol::from_str("tokens"), &tokens);
    }

    fn balance_of(&self, env: Env, account: Address) -> BigInt {
        
        unimplemented!() 
    }

    fn transfer(&self, env: Env, to: Address, amount: BigInt) {
        
        unimplemented!()
    }

    
}
