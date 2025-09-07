use curve420::ristretto::RistrettoPoint;
use curve420::curve::G;

fn main() {
    let rp = RistrettoPoint::from_edwards_checked(&*G).expect("G in subgroup");
    let enc = rp.encode();
    let dec = RistrettoPoint::decode(&enc).expect("decode");
    println!("Round-trip ok: {}", rp == dec);
}

