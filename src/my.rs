fn main() {
	let mut a: uint = 1;
	let mut b: uint = 2;
	match (a,b) {
		(0, 0) => {},
		(0, 1) => {},
		(1, 1) => {},
		(1, 1...9) => {println!("hello")},
		(_, _) => {}
	}
}