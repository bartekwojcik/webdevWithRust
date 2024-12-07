mod chapter4;
use chapter4::inside::suck;
fn main() {
    suck();
    let a = chapter4::add(5, 6);
    println!("{a}");
}
