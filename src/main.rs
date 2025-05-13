use formal_systems_toolbox::{logic::*, *};

fn main() {
    fn zero() -> Judgement {
        constant("zero")
    }
    fn succ(n: Judgement) -> Judgement {
        op!("succ", n)
    }
    fn empty() -> Judgement {
        constant("empty")
    }
    fn node(t1: Judgement, t2: Judgement) -> Judgement {
        op!("node", t1, t2)
    }

    {
        let nat = FormalSystem::new(
            vec![
                Rule::taut("zero", op!("nat", zero())),
                Rule::new(
                    "succ",
                    vec![op!("nat", var("n"))],
                    op!("nat", succ(var("n"))),
                ),
                Rule::new(
                    "tree",
                    vec![op!("tree", var("a1")), op!("tree", var("a2"))],
                    op!("tree", op!("node", var("a1"), var("a2"))),
                ),
                Rule::taut("empty", op!("tree", constant("empty"))),
                Rule::taut("s1", op!("sum", var("n"), zero(), var("n"))),
                Rule::new(
                    "s2",
                    vec![op!("sum", var("n"), var("m"), var("p"))],
                    op!("sum", var("n"), succ(var("m")), succ(var("p"))),
                ),
                Rule::taut("max1", op!("max", var("n"), zero(), var("n"))),
                Rule::taut("max2", op!("max", zero(), var("n"), var("n"))),
                Rule::new(
                    "max3",
                    vec![op!("max", var("n"), var("m"), var("p"))],
                    op!("max", succ(var("n")), succ(var("m")), succ(var("p"))),
                ),
                Rule::taut("h1", op!("hgt", constant("empty"), zero())),
                Rule::new(
                    "h2",
                    vec![
                        op!("hgt", var("t1"), var("n1")),
                        op!("hgt", var("t2"), var("n2")),
                        op!("max", var("n1"), var("n2"), var("n")),
                    ],
                    op!("hgt", op!("node", var("t1"), var("t2")), succ(var("n"))),
                ),
            ],
            8,
        );

        let proof = nat.verify(&op!("nat", succ(zero()))).unwrap();
        println!("Test:{}", proof.to_string_tree());

        let proof = nat.verify(&op!("sum", zero(), zero(), zero())).unwrap();
        println!("Test:{}", proof.to_string_tree());

        let proof = nat
            .verify(&op!(
                "sum",
                succ(zero()),
                succ(succ(zero())),
                succ(succ(succ(zero())))
            ))
            .unwrap();
        println!("Test:{}", proof.to_string_tree());

        let proof = nat
            .verify(&op!(
                "sum",
                succ(zero()),
                var("x"),
                succ(succ(succ(zero())))
            ))
            .unwrap();
        println!("Calc:{}", proof.to_string_tree());

        let proof = nat
            .verify(&op!(
                "max",
                succ(zero()),
                succ(succ(zero())),
                succ(succ(zero()))
            ))
            .unwrap();
        println!("Test:{}", proof.to_string_tree());

        let proof = nat
            .verify(&op!(
                "max",
                succ(succ(succ(zero()))),
                succ(succ(zero())),
                succ(succ(succ(zero())))
            ))
            .unwrap();
        println!("Test:{}", proof.to_string_tree());

        let proof = nat
            .verify(&op!("hgt", node(empty(), empty()), succ(zero())))
            .unwrap();
        println!("Test:{}", proof.to_string_tree());

        let proof = nat
            .verify(&op!(
                "hgt",
                node(empty(), node(empty(), empty())),
                succ(succ(zero()))
            ))
            .unwrap();
        println!("Test:{}", proof.to_string_tree());

        let proof = nat
            .verify(&op!("hgt", node(empty(), node(empty(), empty())), var("x")))
            .unwrap();
        println!("Calc:{}", proof.to_string_tree());

        let proof = nat
            .verify(&op!("hgt", var("x"), succ(succ(zero()))))
            .unwrap();
        println!("Const:{}", proof.to_string_tree());
    }
}
