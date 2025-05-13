# Unravelling Foundations

A basic theorem prover/assistant developed to follow and understand the concepts in the book [Practical Foundations for Programming Languages](https://www.cs.cmu.edu/~rwh/pfpl/). The program allows to declare a formal system with a set of hypotheses and can be asked to verify subsequent judgements, printing a demonstration:

```rust
    use formal_systems_toolbox::{logic::*, *};

    let nat = FormalSystem::new(
        vec![
            Rule::taut("zero", op!("nat", constant("zero"))),
            Rule::new(
                "succ",
                vec![op!("nat", var("n"))],
                op!("nat", op!("succ", var("n"))),
            ),
        ],
        8, // Maximum depth for a derivation tree
    );

    let proof = nat.verify(&op!("nat", succ(zero()))).unwrap();
    println!("Test: {}", proof.to_string_tree());
```

The result will be:

```sh
Test:
   zero---------------   
         nat(zero())     
succ---------------------
      nat(succ(zero()))  
```

The system simply tries every possible derivation path while unifying rules with the current branch. It can diverge, so a maximum depth for derivation is given.

## TODO

 - [x] Implement an automatic theorem prover (brute force)
 - [ ] Parse a proof language
 - [ ] Implement a theorem assistant that proposes the possible paths
 - [ ] Figure out how to handle symbol sets for bindings
