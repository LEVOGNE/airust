#![allow(dead_code)]
mod previous;
mod current;
fn main() {
    let a=std::env::args().collect::<Vec<_>>();
    let width=a[2].parse::<u32>().unwrap();
    match a[1].as_str() { "previous"=>previous::probe(width,&a[3]),"current"=>current::probe(width,&a[3]),_=>panic!("unknown version") }
    println!("validated:{}:{}:{}",a[1],width,a[3]);
}
