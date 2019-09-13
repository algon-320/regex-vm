mod regex;

fn print_instructions(ins: &regex::Ins) {
    for (idx, instruction) in ins.iter().enumerate() {
        println!("{:02}: {:?}", idx, instruction);
    }
}

fn main() {
    // let pat = r#"^a?b+[cde]{1,3}|hoge(.+)|(te(s)?t)$"#;
    let pat = r#"^(a(bra)?(cad)?)+$"#;
    let text_set = vec![
        // r#"abc"#,
        // r#"bbcc"#,
        // r#"ac"#,
        // r#"bcde"#,
        // r#"xyz"#,
        // r#"hogeXXXX"#,
        r#"abracadabra"#,
        r#"abraabra"#,
        r#"abra"#,
        r#"cadcad"#,
    ];

    println!("Pattern: {}", pat);
    let ast = regex::parse(pat);
    // println!("--------------------------------");
    // println!("AST: {:?}", ast);
    let ins = regex::compile(ast.unwrap()).unwrap();
    // println!("--------------------------------");
    // print_instructions(&ins);

    for text in text_set.iter() {
        println!("--------------------------------");
        println!("Text: {}", text);
        println!("Result: {:?}", regex::search(&ins, text.to_string()));
    }
}
