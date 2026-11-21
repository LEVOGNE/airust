pub fn probe(width: u32, mode: &str) {
    use std::collections::HashMap;
    let limits = Limits { depth: 50, references: 1_000_000, pages: if mode == "prefix" { Some(1) } else { None } };
    let result = count_tree(ObjectRef { id: 1, gen: 0 }, limits, |r| {
        Ok(if r.id == 1 {
            Object::Dictionary(HashMap::from([
                ("Type".into(), Object::Name("Pages".into())),
                ("Count".into(), Object::Integer(i64::from(width))),
                ("Kids".into(), Object::Array((2..width+2).map(|id| Object::Reference(ObjectRef {id, gen:0})).collect())),
            ]))
        } else {
            Object::Dictionary(HashMap::from([("Type".into(), Object::Name("Page".into()))]))
        })
    });
    if mode == "prefix" { assert!(result.unwrap_err().contains("page count (2; maximum 1)")); }
    else { assert_eq!(result.unwrap(), width as usize); }
}
