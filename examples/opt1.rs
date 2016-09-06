
extern crate mostinefficientsha;

use mostinefficientsha::linopt::Linopt;
use mostinefficientsha::util::hex;

fn main() {

    let l = Linopt::new(64, "87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7");
    println!("{:?}", l);
    l.init();
    l.optimize(1);
    println!("{}", hex(&l.eval_to_u32()[..]));

}