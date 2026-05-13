mod node;
mod parser;

use node::*;
use parser::*;

fn main() {
    let mut node = LeptValue::default();
    LeptParser::parse(
        &mut node,
        "{ \"a\": 1, \"b\": 2, \"c\": [1, true, false, \"hello\", null, 4] }",
    );
    println!("{}", node);

    let s = node.to_string();
    let status = LeptParser::parse(&mut node, s.as_str());
    println!("{:?}", status);
}
