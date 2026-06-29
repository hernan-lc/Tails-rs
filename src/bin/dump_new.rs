use tails::compiler::bytecode::generate;
use tails::compiler::lexer::tokenize;
use tails::compiler::parser::parse;

fn main() {
    let source = r#"
        function Foo(x) {
            this.x = x;
        }
        let f = new Foo(42);
        f.x;
    "#;
    let mut tokens = tokenize(source).unwrap();
    let ast = parse(&mut tokens).unwrap();
    let module = generate(&ast).unwrap();
    for (idx, instr) in module.instructions.iter().enumerate() {
        println!("{:4}: {:?}", idx, instr);
    }
}
